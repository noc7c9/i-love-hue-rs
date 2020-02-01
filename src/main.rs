use rand::random;
use stdweb::js;
use web_logger;
use yew::{html, Callback, ClickEvent, Component, ComponentLink, Html, ShouldRender};

mod gradient;
mod grid;
mod puzzle;
mod puzzle_view;

use puzzle_view::PuzzleView;

enum GameState {
    Initial,
    Playing,
    GameOver,
}

struct App {
    link: ComponentLink<Self>,
    state: GameState,
    puzzle: puzzle::Puzzle,
}

enum Msg {
    StartGame,
    CompleteGame,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        use stdweb::unstable::TryInto;

        let win_width = js! {
            return window.innerWidth || document.documentElement.clientWidth || document.body.clientWidth;
        }.try_into().expect("Failed to get window height");
        let win_height = js! {
            return window.innerHeight|| document.documentElement.clientHeight|| document.body.clientHeight;
        }.try_into().expect("Failed to get window height");
        log::info!("{:?},{:?}", win_width, win_height);

        App {
            link,
            state: GameState::Initial,
            puzzle: puzzle::generate_lvl1_puzzle((win_width, win_height)),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::StartGame => {
                self.puzzle.seed = random();
                self.state = GameState::Playing
            }
            Msg::CompleteGame => self.state = GameState::GameOver,
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <>
                <PuzzleView puzzle=self.puzzle.clone() oncomplete=self.link.callback(|_| Msg::CompleteGame) />
                {
                    match self.state {
                        GameState::Initial => start_game_ui_overlay(self.link.callback(|_| Msg::StartGame)),
                        GameState::Playing => html!{},
                        GameState::GameOver => game_over_ui_overlay(self.link.callback(|_| Msg::StartGame)),
                    }
                }
            </>
        }
    }
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

    yew::start_app::<App>();
}
