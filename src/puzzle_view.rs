use crate::debug;
use crate::gradient::{Color, Position};
use crate::grid::Grid;
use crate::puzzle::Puzzle;
use rand::prelude::*;
use yew::prelude::*;

pub struct PuzzleView {
    props: Props,
    link: ComponentLink<Self>,
    grid: Grid<Cell>,
    active_tile: Option<usize>,
}

struct Cell {
    index: usize,
    color: Color,
    is_locked: bool,
}

pub enum Msg {
    TouchTile(usize),
}

#[derive(Clone, Properties)]
pub struct Props {
    #[props(required)]
    pub puzzle: Puzzle,
    #[props(required)]
    pub oncomplete: Callback<()>,
}

impl Component for PuzzleView {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let grid = grid_from_puzzle(&props.puzzle);
        Self {
            props,
            link,
            grid,
            active_tile: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::TouchTile(index) => {
                // ignore locked tiles
                if self.grid.get(index).is_locked {
                    return false;
                }

                if let Some(active_tile) = self.active_tile {
                    if active_tile != index {
                        self.grid.swap(active_tile, index);
                        if is_grid_in_order(&self.grid) {
                            self.props.oncomplete.emit(());
                        }
                    }
                    self.active_tile = None;
                } else {
                    self.active_tile = Some(index);
                }
                true
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.props.puzzle != props.puzzle {
            self.grid = grid_from_puzzle(&props.puzzle);
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let (width, height) = self.grid.dims();

        html! {
            <div
                class="grid"
                style=format!("--grid-width: {}; --grid-height: {}", width, height)>
            {
                self.grid.iter().enumerate()
                    .map(|(i, cell)| {
                        let is_active = Some(i) == self.active_tile;
                        let onclick = self.link.callback(move |_| Msg::TouchTile(i));
                        color_tile(cell, is_active, onclick)
                    })
                    .collect::<Html>()
            }
            </div>
        }
    }
}

fn grid_from_puzzle(puzzle: &Puzzle) -> Grid<Cell> {
    let Puzzle {
        width,
        height,
        seed,
        ..
    } = *puzzle;

    let mut grid = Grid::from_closure(width, height, |x, y| {
        let x_off = x as f64 / (width as f64 - 1.0);
        let y_off = y as f64 / (height as f64 - 1.0);
        Cell {
            index: y * width + x,
            color: puzzle.gradient.color_at(Position::new(x_off, y_off)),
            is_locked: puzzle.locked.contains(&(x, y)),
        }
    });

    if !debug::disable_shuffle() {
        shuffle_grid(&mut grid, seed);
    }

    grid
}

fn is_grid_in_order(grid: &Grid<Cell>) -> bool {
    let mut iter = grid.iter();
    let mut prev = iter.next().map(|cell| cell.index);
    for cell in iter {
        let this = Some(cell.index);
        if prev > this {
            return false;
        }
        prev = this;
    }
    true
}

fn shuffle_grid(grid: &mut Grid<Cell>, seed: u64) {
    let unlocked_tiles = grid
        .iter()
        .enumerate()
        .filter_map(|(idx, cell)| if cell.is_locked { None } else { Some(idx) })
        .collect::<Vec<usize>>();
    let mut shuffled = unlocked_tiles.clone();

    shuffled.shuffle(&mut rand_pcg::Pcg64Mcg::seed_from_u64(seed));

    for (original_position, shuffled_position) in unlocked_tiles.into_iter().zip(shuffled) {
        grid.swap(original_position, shuffled_position);
    }

    if is_grid_in_order(&grid) {
        shuffle_grid(grid, seed + 1)
    }
}

fn color_tile(cell: &Cell, is_active: bool, onclick: Callback<ClickEvent>) -> Html {
    let class = match (is_active, cell.is_locked) {
        (true, true) => "cell active locked",
        (true, false) => "cell active interactive",
        (false, true) => "cell locked",
        (false, false) => "cell interactive",
    };
    let style = format!("background: {}", cell.color.to_css());
    html! {
        <div class=class onclick=onclick>
            <div class="tile" style=style>
                {
                    if cell.is_locked {
                        html! {<div class="lock" />}
                    } else if debug::show_cell_numbers() {
                        html! {<div>{cell.index}</div>}
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}
