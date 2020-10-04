use wasm_bindgen::prelude::*;

use engine::{
    Game, GameRun, GameState, Resources,
    sound::Music,
    util::setup_panic_hook,
};
use states::main_menu::MainMenuState;

use crate::engine::sound::Sound;

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
            background: resources.load_music("assets/background.mp3").looped(),
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
}

impl Game for QuantumLoops {
    fn load(resources: Resources) -> (Self, Box<dyn GameState<QuantumLoops>>) {
        let global = QuantumLoops {
            sounds: Sounds::load(resources)
        };
        (global, Box::new(MainMenuState::new()))
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    QuantumLoops::run();
}
