use log::*;
use wasm_bindgen::prelude::*;

use engine::{
    *,
    event::Event::MouseMove,
    sprite::Sprite,
};
use util::setup_panic_hook;

use crate::engine::event::Event::MouseDown;
use crate::engine::sound::Sound;
use crate::engine::event::MouseButton;

mod engine;

struct MyGame {
    test: [Sprite; 3],
    red_pos: [f64; 2],
    sound: Sound,
}

impl engine::Game for MyGame {
    fn load(resources: Resources) -> Self {
        let spritesheet = resources.load_spritesheet("assets/spritesheet.png");
        MyGame {
            // sound: resources.load_sound("https://upload.wikimedia.org/wikipedia/commons/f/f2/Median_test.ogg"),
            sound: resources.load_sound("assets/blip.wav"),
            red_pos: [0.0, 0.0],
            test: [
                spritesheet.create_sprite(0, 0, 11, 11),
                spritesheet.create_sprite(11, 0, 11, 11),
                spritesheet.create_sprite(0, 11, 11, 11),
            ],
        }
    }

    fn update(&mut self, mut context: GameUpdate) {
        for event in context.events() {
            match event {
                MouseMove { x, y, .. } => {
                    self.red_pos = [x as f64, y as f64];
                }
                MouseDown { button: MouseButton::Right, .. } => {
                    if self.sound.playing() {
                        self.sound.stop();
                        debug!("STOPPING");
                    } else {
                        self.sound.play();
                        debug!("STARTING");
                    }
                }
                _ => {}
            }
        }

        let [x, y] = self.red_pos;

        self.test[0].draw(30.0, 10.0);
        self.test[2].draw(50.0, 10.0);
        self.test[1].draw(x - 5.0, y - 5.0);
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    info!("main function called");

    MyGame::run();
}
