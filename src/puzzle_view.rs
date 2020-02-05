use crate::debug;
use crate::puzzle::{Puzzle, PuzzleCell};
use crate::savegame;
use crate::SAVEGAME_KEY;
use yew::prelude::*;

pub struct PuzzleView {
    props: Props,
    link: ComponentLink<Self>,
    active_tile: Option<usize>,
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
        Self {
            props,
            link,
            active_tile: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::TouchTile(index) => {
                // ignore locked tiles
                if self.props.puzzle.get(index).is_locked {
                    return false;
                }

                if let Some(active_tile) = self.active_tile {
                    if active_tile != index {
                        self.props.puzzle.swap(active_tile, index);
                        savegame::save(SAVEGAME_KEY, &self.props.puzzle);
                        if self.props.puzzle.is_solved() {
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
        if self.props.puzzle.settings != props.puzzle.settings {
            self.props = props;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        let (width, height) = self.props.puzzle.dimensions();

        html! {
            <div
                class="grid"
                style=format!("--grid-width: {}; --grid-height: {}", width, height)>
            {
                self.props.puzzle.iter().enumerate()
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

fn color_tile(cell: &PuzzleCell, is_active: bool, onclick: Callback<ClickEvent>) -> Html {
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
                        html! {<div>{cell.solved_position}</div>}
                    } else {
                        html! {}
                    }
                }
            </div>
        </div>
    }
}
