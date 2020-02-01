use crate::gradient::{Color, Gradient};

#[derive(Debug, Clone, PartialEq)]
pub struct Puzzle {
    pub width: usize,
    pub height: usize,
    pub gradient: Gradient,
    pub locked: Vec<(usize, usize)>,
    pub seed: u64,
}

pub fn generate_lvl1_puzzle((win_width, win_height): (usize, usize)) -> Puzzle {
    // Dynamically size the initial puzzle based on the window size so that
    // the shorter side has 5 cells and
    // the cells are square-ish
    let ratio = win_width.max(win_height) as f64 / win_width.min(win_height) as f64;
    let short = 5;

    let long = 2.max((short as f64 * ratio).round() as usize);
    let (width, height) = if win_width < win_height {
        (short, long)
    } else {
        (long, short)
    };

    generate(width, height)
}

pub fn generate(width: usize, height: usize) -> Puzzle {
    Puzzle {
        width,
        height,
        gradient: Gradient::builder()
            .top_left(Color::rgb(31, 94, 203))
            .top_right(Color::rgb(236, 130, 130))
            .bottom_left(Color::rgb(50, 188, 246))
            .bottom_right(Color::rgb(219, 231, 66))
            .build(),
        locked: vec![
            (0, 0),
            (width - 1, 0),
            (0, height - 1),
            (width - 1, height - 1),
        ],
        seed: rand::random(),
    }
}
