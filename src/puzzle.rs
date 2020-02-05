use crate::debug;
use crate::gradient::{Color, Gradient, Position};
use crate::grid::{Grid, Iter as GridIter};
use lazy_static::lazy_static;
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand_distr::Normal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PuzzleSettings {
    difficulty: usize,
    width: usize,
    height: usize,
    gradient: Gradient,
    locking_pattern: LockingPattern,
    shuffle_seed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Puzzle {
    pub settings: PuzzleSettings,
    grid: Grid<PuzzleCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PuzzleCell {
    pub solved_position: usize,
    pub is_locked: bool,
    pub color: Color,
}

impl Puzzle {
    pub fn generate_lvl1(win_size: (usize, usize)) -> Self {
        let difficulty = debug::starting_difficulty().unwrap_or(1);
        let settings = PuzzleSettings::from_difficulty(difficulty, win_size);
        Self::from_settings(settings)
    }

    pub fn next_level(&mut self, win_size: (usize, usize)) {
        let difficulty = self.settings.difficulty + 1;
        let settings = PuzzleSettings::from_difficulty(difficulty, win_size);
        *self = Self::from_settings(settings);
    }

    fn from_settings(settings: PuzzleSettings) -> Self {
        let PuzzleSettings { width, height, .. } = settings;

        let grid = Grid::from_closure(width, height, |x, y| PuzzleCell {
            solved_position: y * width + x,
            is_locked: settings.is_cell_locked(x, y),
            color: settings.get_cell_color(x, y),
        });

        let mut puzzle = Self { settings, grid };

        if !debug::disable_shuffle() {
            puzzle.shuffle();
        }

        puzzle
    }

    fn shuffle(&mut self) {
        let unlocked_tiles = self
            .grid
            .iter()
            .enumerate()
            .filter_map(|(idx, cell)| if cell.is_locked { None } else { Some(idx) })
            .collect::<Vec<usize>>();
        let mut shuffled = unlocked_tiles.clone();

        shuffled.shuffle(&mut rand_pcg::Pcg64Mcg::seed_from_u64(
            self.settings.shuffle_seed,
        ));

        for (original_position, shuffled_position) in unlocked_tiles.into_iter().zip(shuffled) {
            self.grid.swap(original_position, shuffled_position);
        }

        // If the shuffled puzzle is solved, reshuffled with the next seed
        if self.is_solved() {
            self.settings.shuffle_seed += 1;
            self.shuffle();
        }
    }

    pub fn dimensions(&self) -> (usize, usize) {
        self.grid.dims()
    }

    pub fn get(&self, index: usize) -> &PuzzleCell {
        self.grid.get(index)
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        self.grid.swap(a, b)
    }

    pub fn iter(&self) -> GridIter<PuzzleCell> {
        self.grid.iter()
    }

    pub fn is_solved(&self) -> bool {
        let mut iter = self.grid.iter();
        let mut prev = iter.next().map(|cell| cell.solved_position);
        for cell in iter {
            let this = Some(cell.solved_position);
            if prev > this {
                return false;
            }
            prev = this;
        }
        true
    }
}

impl PuzzleSettings {
    fn from_difficulty(difficulty: usize, win_size: (usize, usize)) -> Self {
        let (width, height) = generate_puzzle_size(difficulty, win_size);
        Self {
            difficulty,
            width,
            height,
            gradient: generate_gradient(difficulty),
            locking_pattern: generate_locking_pattern(difficulty),
            shuffle_seed: random(),
        }
    }

    fn get_cell_color(&self, x: usize, y: usize) -> Color {
        let x_off = x as f64 / (self.width as f64 - 1.0);
        let y_off = y as f64 / (self.height as f64 - 1.0);
        self.gradient.color_at(Position::new(x_off, y_off))
    }

    fn is_cell_locked(&self, x: usize, y: usize) -> bool {
        use LockingPattern::*;

        let is_corner = || (x == 0 || x == (self.width - 1)) && (y == 0 || y == (self.height - 1));
        let is_border = || x == 0 || x == (self.width - 1) || y == 0 || y == (self.height - 1);
        let is_checkboard = || (x + y) % 2 == 0;
        let is_shortlines = || (if self.width > self.height { x } else { y }) % 2 == 0;
        let is_longlines = || (if self.width < self.height { x } else { y }) % 2 == 0;

        match self.locking_pattern {
            Corners => is_corner(),
            Borders => is_border(),
            ReverseBorders => !is_border(),
            CheckerboardA => is_checkboard(),
            CheckerboardB => !is_checkboard(),
            HalfCheckerboardA => x % 2 == 0 && y % 2 == 0,
            HalfCheckerboardB => x % 2 != 0 && y % 2 != 0,
            ShortLinesA => is_shortlines(),
            ShortLinesB => !is_shortlines(),
            LongLinesA => is_longlines(),
            LongLinesB => !is_longlines(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
enum LockingPattern {
    Corners,
    Borders,
    ReverseBorders,
    CheckerboardA,
    CheckerboardB,
    HalfCheckerboardA,
    HalfCheckerboardB,
    ShortLinesA,
    ShortLinesB,
    LongLinesA,
    LongLinesB,
}

fn generate_puzzle_size(
    difficulty: usize,
    (win_width, win_height): (usize, usize),
) -> (usize, usize) {
    // Dynamically size the initial puzzle based on the window size so that
    // the shorter side has X cells and
    // the cells are square-ish
    //
    // X is based on the difficulty with higher difficulties creating bigger puzzles
    const MIN_CELLS: usize = 5;
    let short = MIN_CELLS + (difficulty as f64).log(4.0).powf(2.0).trunc() as usize;

    let ratio = win_width.max(win_height) as f64 / win_width.min(win_height) as f64;
    let long = 2.max((short as f64 * ratio).round() as usize);

    if win_width < win_height {
        (short, long)
    } else {
        (long, short)
    }
}

fn generate_gradient(difficulty: usize) -> Gradient {
    // Higher difficulties create gradients with a lower hue variance
    const MAX_HUE: f64 = 300.0;
    const MIN_HUE: f64 = 90.0;
    let hue_variance = (MAX_HUE - (difficulty - 1) as f64).max(MIN_HUE);

    debug_assert!(hue_variance > 0.0 && hue_variance <= 360.0);

    let mut rng = thread_rng();

    let start_hue = rng.gen_range(0.0, 360.0);
    let diff = hue_variance / 4.0;
    let hues = [
        start_hue,
        start_hue + diff,
        start_hue + diff * 2.0,
        start_hue + diff * 3.0,
    ];

    fn sample_ranged_normal(mean: f64, sd: f64, min: f64, max: f64) -> f64 {
        let mut rng = thread_rng();
        loop {
            let value = rng.sample(Normal::new(mean, sd).unwrap());
            if value >= min && value <= max {
                return value;
            }
        }
    }

    let gen_s = || sample_ranged_normal(0.9, 0.1, 0.5, 1.0);
    let gen_l = || sample_ranged_normal(0.5, 0.005, 0.4, 0.7);

    let top_left = Color::hsl(hues[0], gen_s(), gen_l());
    let top_right = Color::hsl(hues[1], gen_s(), gen_l());
    let bottom_right = Color::hsl(hues[2], gen_s(), gen_l());
    let bottom_left = Color::hsl(hues[3], gen_s(), gen_l());

    Gradient::builder()
        .top_left(top_left.to_rgb())
        .top_right(top_right.to_rgb())
        .bottom_left(bottom_left.to_rgb())
        .bottom_right(bottom_right.to_rgb())
        .build()
}

fn generate_locking_pattern(_difficulty: usize) -> LockingPattern {
    // generate a random locking pattern
    // weighted roughly according to difficulty of the locking pattern
    // harder patterns have a lower weight and are less likely to be selected
    use LockingPattern::*;
    const PATTERNS: [(LockingPattern, usize); 11] = [
        (Corners, 1),
        (Borders, 2),
        (HalfCheckerboardA, 5),
        (HalfCheckerboardB, 5),
        (ShortLinesA, 6),
        (ShortLinesB, 6),
        (LongLinesA, 7),
        (LongLinesB, 7),
        (CheckerboardA, 7),
        (CheckerboardB, 7),
        (ReverseBorders, 2), // lower because this pattern is too easy
    ];
    lazy_static! {
        static ref DISTRIBUTION: WeightedIndex<usize> =
            WeightedIndex::new(PATTERNS.iter().map(|item| item.1)).unwrap();
    }
    PATTERNS[DISTRIBUTION.sample(&mut thread_rng())].0
}
