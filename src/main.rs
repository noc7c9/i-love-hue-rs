use stdweb::js;
use web_logger;
use yew::{html, Callback, ClickEvent, Component, ComponentLink, Html, ShouldRender};

mod debug;
mod gradient;
mod grid;
mod puzzle;
mod puzzle_view;
mod savegame;

use puzzle::Puzzle;
use puzzle_view::PuzzleView;

pub const SAVEGAME_KEY: &str = "SAVEGAME";

enum GameState {
    Initial,
    Playing,
    GameOver,
}

struct App {
    link: ComponentLink<Self>,
    state: GameState,
    puzzle: Puzzle,
}

enum Msg {
    StartGame,
    CompletePuzzle,
    NextLevel,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let win_size = get_win_size();
        let puzzle = if let Some(mut puzzle) = savegame::load::<Puzzle>(SAVEGAME_KEY) {
            // if the loaded puzzle is already solved
            if puzzle.is_solved() {
                // go to the next level
                puzzle.next_level(win_size);
                savegame::save(SAVEGAME_KEY, &puzzle);
            }
            puzzle
        } else {
            let puzzle = Puzzle::generate_lvl1(win_size);
            savegame::save(SAVEGAME_KEY, &puzzle);
            puzzle
        };
        App {
            link,
            state: GameState::Initial,
            puzzle,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartGame => self.state = GameState::Playing,
            Msg::NextLevel => {
                let win_size = get_win_size();
                self.puzzle.next_level(win_size);
                savegame::save(SAVEGAME_KEY, &self.puzzle);
                self.state = GameState::Playing
            }
            Msg::CompletePuzzle => self.state = GameState::GameOver,
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <PuzzleView puzzle=self.puzzle.clone() oncomplete=self.link.callback(|_| Msg::CompletePuzzle) />
                {
                    match self.state {
                        GameState::Initial => start_game_ui_overlay(self.link.callback(|_| Msg::StartGame)),
                        GameState::Playing => html!{},
                        GameState::GameOver => game_over_ui_overlay(self.link.callback(|_| Msg::NextLevel)),
                    }
                }
            </>
        }
    }
}

fn get_win_size() -> (usize, usize) {
    use stdweb::unstable::TryInto;

    let win_width = js! {
            return window.innerWidth || document.documentElement.clientWidth || document.body.clientWidth;
        }.try_into().expect("Failed to get window height");
    let win_height = js! {
            return window.innerHeight|| document.documentElement.clientHeight|| document.body.clientHeight;
        }.try_into().expect("Failed to get window height");
    (win_width, win_height)
}

fn start_game_ui_overlay(onclick: Callback<ClickEvent>) -> Html {
    html! {
        <div class="ui-overlay">
            <div class="ui-text" onclick=onclick>{"Start"}</div>
        </div>
    }
}

fn game_over_ui_overlay(onclick: Callback<ClickEvent>) -> Html {
    html! {
        <div class="ui-overlay">
            <div class="ui-text" onclick=onclick>{"Play Again"}</div>
        </div>
    }
}

fn main() {
    web_logger::init();
    debug::init();
    yew::start_app::<App>();
}
