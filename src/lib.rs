use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::Response;

use crate::engine::util::{Mut, Bitmap};
use engine::{sound::Sound, util::setup_panic_hook, window, Game, GameRun, GameState, Resources};
use level::{GameLevel, StoredData};
use states::main_menu::MainMenuState;

mod engine;
mod level;
mod states;

#[derive(Debug)]
pub struct Sounds {
    background: Sound,
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
            background: resources
                .load_sound("assets/background.mp3")
                .with_volume(0.01)
                .with_layers(Bitmap::empty().with_on(1))
                .looped(),
            win: resources.load_sound("assets/win.wav").with_volume(0.2),
            lose: resources.load_sound("assets/lose.wav").with_volume(0.2),
            hover: resources.load_sound("assets/hover.wav").with_volume(0.2),
            click: resources.load_sound("assets/click.wav").with_volume(0.2),
            jump: resources.load_sound("assets/jump.wav").with_volume(0.2),
            wrong_ring: resources
                .load_sound("assets/wrong-ring.wav")
                .with_volume(0.2),
        }
    }
}

#[derive(Debug)]
pub struct QuantumLoops {
    sounds: Sounds,
    levels: Mut<Option<Vec<GameLevel>>>,
}

impl QuantumLoops {
    pub fn sounds(&self) -> &Sounds {
        &self.sounds
    }

    pub fn get_level(&self, level: usize) -> Option<GameLevel> {
        self.levels
            .borrow_mut()
            .as_ref()
            .and_then(|levels| levels.get(level).cloned())
    }

    pub fn level_count(&self) -> usize {
        self.levels
            .borrow()
            .as_ref()
            .map(Vec::len)
            .unwrap_or_default()
    }

    fn load_levels(&self) {
        let moved_levels = self.levels.clone();
        spawn_local(async move {
            *moved_levels.borrow_mut() = Some(
                async move {
                    let response: Response =
                        JsFuture::from(window().fetch_with_str("assets/levels.json"))
                            .await?
                            .dyn_into()?;
                    JsFuture::from(response.json()?).await
                }
                .await
                .unwrap()
                .into_serde()
                .unwrap(),
            );
        });
    }
}

impl Game for QuantumLoops {
    type Storage = StoredData;

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
