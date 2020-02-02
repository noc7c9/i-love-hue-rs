use crate::debug;
use crate::gradient::{Color, Gradient, Position};
use rand::prelude::*;
use rand_distr::Normal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Puzzle {
    difficulty: usize,
    pub width: usize,
    pub height: usize,
    gradient: Gradient,
    locking_pattern: LockingPattern,
    pub shuffle_seed: u64,
}

impl Puzzle {
    pub fn generate_lvl1(win_size: (usize, usize)) -> Self {
        Self::from_difficulty(debug::starting_difficulty().unwrap_or(1), win_size)
    }

    pub fn next_level(&mut self, win_size: (usize, usize)) {
        *self = Self::from_difficulty(self.difficulty + 1, win_size);
    }

    fn from_difficulty(difficulty: usize, win_size: (usize, usize)) -> Self {
        log::info!("difficulty = {}", difficulty);
        let (width, height) = generate_puzzle_size(difficulty, win_size);
        Self {
            difficulty,
            width,
            height,
            gradient: generate_gradient(difficulty),
            locking_pattern: LockingPattern::Corners,
            shuffle_seed: random(),
        }
    }

    pub fn get_cell_color(&self, x: usize, y: usize) -> Color {
        let x_off = x as f64 / (self.width as f64 - 1.0);
        let y_off = y as f64 / (self.height as f64 - 1.0);
        self.gradient.color_at(Position::new(x_off, y_off))
    }

    pub fn is_cell_locked(&self, x: usize, y: usize) -> bool {
        match self.locking_pattern {
            LockingPattern::Corners => {
                (x == 0 || x == (self.width - 1)) && (y == 0 || y == (self.height - 1))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LockingPattern {
    Corners,
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
    log::info!("short = {}", short);

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
    log::info!("hue variance = {}", hue_variance);

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

    // log::info!("\nstart hue = {}\nend hue = {}({})\n\ntop left = {:?}\ntop right = {:?}\nbottom left = {:?}\nbottom right = {:?}", start_hue, start_hue + hue_variance, start_hue + diff * 3.0, top_left, top_right, bottom_left, bottom_right);

    Gradient::builder()
        .top_left(top_left.to_rgb())
        .top_right(top_right.to_rgb())
        .bottom_left(bottom_left.to_rgb())
        .bottom_right(bottom_right.to_rgb())
        .build()
}
