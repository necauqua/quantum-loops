use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::Response;

use engine::{
    Game, GameRun, GameState, Resources,
    sound::Music,
    util::setup_panic_hook,
};
use states::main_menu::MainMenuState;

use crate::engine::sound::Sound;
use crate::level::{GameLevel, StoredData};
use crate::engine::window;
use wasm_bindgen::JsCast;

mod engine;
mod states;
mod ui;
mod level;

// waiting for that (or at least for TWO_PI lol)
pub const TAU: f64 = 2.0 * std::f64::consts::PI;

pub struct Sounds {
    background: Music,
    win: Sound,
    lose: Sound,
    hover: Sound,
    click: Sound,
    jump: Sound,
    wrong_ring: Sound,
}

impl Sounds {
    fn load(resources: Resources) -> Sounds {
        Self {
            background: resources.load_music("assets/background.mp3")
                .with_volume(0.01)
                .looped(),
            win: resources.load_sound("assets/win.wav").with_volume(0.2),
            lose: resources.load_sound("assets/lose.wav").with_volume(0.2),
            hover: resources.load_sound("assets/hover.wav").with_volume(0.2),
            click: resources.load_sound("assets/click.wav").with_volume(0.2),
            jump: resources.load_sound("assets/jump.wav").with_volume(0.2),
            wrong_ring: resources.load_sound("assets/wrong-ring.wav").with_volume(0.2),
        }
    }
}

pub struct QuantumLoops {
    sounds: Sounds,
    levels: Rc<RefCell<Option<Vec<GameLevel>>>>,
}

pub fn sounds_enabled() -> bool {
    engine::get_data::<StoredData>().sounds_enabled
}

pub fn music_enabled() -> bool {
    engine::get_data::<StoredData>().music_enabled
}

impl QuantumLoops {
    pub fn sounds(&self) -> &Sounds {
        &self.sounds
    }

    pub fn get_level(&mut self, level: usize) -> Option<GameLevel> {
        match &*self.levels.borrow_mut() {
            Some(levels) => levels.get(level).cloned(),
            _ => None,
        }
    }

    pub fn level_count(&self) -> usize {
        self.levels.borrow()
            .as_ref()
            .map(Vec::len)
            .unwrap_or_default()
    }

    fn load_levels(&self) {
        let moved_levels = self.levels.clone();
        spawn_local(async move {
            *moved_levels.borrow_mut() = Some(async move {
                let response: Response = JsFuture::from(window()
                    .fetch_with_str("assets/levels.json"))
                    .await?
                    .dyn_into()?;
                JsFuture::from(response.json()?)
                    .await
            }.await
                .unwrap()
                .into_serde()
                .unwrap());
        });
    }
}

impl Game for QuantumLoops {
    fn load(resources: Resources) -> (Self, Box<dyn GameState<QuantumLoops>>) {
        let levels = Default::default();
        let global = QuantumLoops {
            sounds: Sounds::load(resources),
            levels,
        };
        global.load_levels();
        (global, Box::new(MainMenuState::new()))
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    QuantumLoops::run();
}
