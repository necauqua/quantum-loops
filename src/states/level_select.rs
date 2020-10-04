use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{*, prelude::*};
use web_sys::Response;

use crate::{
    engine::{
        event::Event,
        GameState,
        GameUpdate,
        StateTransition,
        util::PromiseGlue,
        util::RemConversions,
        window
    },
    QuantumLoops,
    states::main_menu::Background,
    ui::Buttons,
    level::GameLevel,
};
use crate::states::main_game::MainGameState;
use crate::states::main_menu::MainMenuState;

pub struct LevelSelectState {
    levels: Rc<RefCell<Option<Vec<GameLevel>>>>,
    levels_added: bool,
    background: Background,
    buttons: Buttons,
    button_scroll: f32,
    button_limit: f32,
}

impl LevelSelectState {
    pub fn new() -> Self {
        Self {
            levels: Default::default(),
            levels_added: false,
            background: Background::new(),
            buttons: Buttons::new(),
            button_scroll: 0.0,
            button_limit: 0.0,
        }
    }
}

#[wasm_bindgen(inline_js = "\
export function load_json(url) {
    return fetch(url).then(res => res.json());
}")]
extern "C" {
    fn load_json(url: &str) -> js_sys::Promise;
}

impl GameState<QuantumLoops> for LevelSelectState {
    fn on_mounted(&mut self, _: &mut GameUpdate<QuantumLoops>) {
        let moved_levels = self.levels.clone();
        let _ = load_json("assets/levels.json")
            .rust_then(move |json: JsValue| {
                let levels: Vec<GameLevel> = json.into_serde().unwrap();
                *moved_levels.borrow_mut() = Some(levels);
            });
    }

    fn on_event(&mut self, event: Event, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match event {
            Event::MouseWheel { delta, .. } => {
                let yoff = -delta.y * 10.0;
                let new_scroll = self.button_scroll - yoff;

                if new_scroll >= 0.0 && new_scroll < self.button_limit {
                    self.button_scroll = new_scroll;
                    self.buttons.move_buttons([0.0, yoff].into());
                }
                StateTransition::None
            }
            _ => {
                if let Some(i) = self.buttons.on_event(&event, context) {
                    return if i == 0 {
                        StateTransition::Set(Box::new(MainMenuState::new()))
                    } else {
                        let game_level = self.levels.borrow_mut().as_ref().unwrap()[i - 1].clone();
                        StateTransition::Set(Box::new(MainGameState::new(game_level)))
                    }
                }
                StateTransition::None
            }
        }
    }

    fn update(&mut self, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        // levels finally arrived
        if let Some(levels) = self.levels.borrow_mut().as_ref() {
            if !self.levels_added {
                self.levels_added = true;

                let size = context.size();
                let x = size.x * 0.5;
                let off = 2.5.rem_to_pixels() * 1.5;
                let mut y = size.y * 0.25;

                y += off;
                self.buttons.add_button([x, y].into(), "back".into());

                for level in levels {
                    y += off;
                    self.buttons.add_button([x, y].into(), level.name.clone());
                }

                self.button_limit = self.buttons.len() as f32 * off - size.y * 0.25;
            }
        }

        self.background.update(context);
        self.buttons.update(context);
        StateTransition::None
    }
}
