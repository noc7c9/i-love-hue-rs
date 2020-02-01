use stdweb::{js, unstable::TryInto};

pub fn show_cell_numbers() -> bool {
    if cfg!(debug_assertions) {
        (js! { return localStorage.DEBUG_SHOW_CELL_NUMBERS == "true"; })
            .try_into()
            .unwrap()
    } else {
        false
    }
}

pub fn disable_shuffle() -> bool {
    if cfg!(debug_assertions) {
        (js! { return localStorage.DEBUG_DISABLE_SHUFFLE == "true"; })
            .try_into()
            .unwrap()
    } else {
        false
    }
}

pub fn init() {
    if cfg!(debug_assertions) {
        // Setup JS helper functions to toggle debug settings
        js! {
            const toggle = key => () => {
                localStorage[key] = localStorage[key] !== "true";
            };
            window.debug = {
                toggleShowCellNumbers: toggle("DEBUG_SHOW_CELL_NUMBERS"),
                toggleDisableShuffle: toggle("DEBUG_DISABLE_SHUFFLE"),
            }
        }

        // Display the current debug settings
        if show_cell_numbers() {
            log::warn!("DEBUG_SHOW_CELL_NUMBERS is turned on");
        }
        if disable_shuffle() {
            log::warn!("DEBUG_DISABLE_SHUFFLE is turned on");
        }
    }
}
