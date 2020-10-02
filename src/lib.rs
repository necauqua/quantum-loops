use log::*;
use wasm_bindgen::prelude::*;

use engine::{
    *,
    event::Event::MouseMove,
    sprite::Sprite,
    sound::Sound,
};
// <!--    <source src="" type="audio/mpeg">-->

mod engine;

struct MyGame {
    test: [Sprite; 3],
    last_click: [f64; 2],
    sound: Sound,
    playing: bool,
}

impl engine::Game for MyGame {
    fn load(resources: Resources) -> Self {
        let spritesheet = resources.load_spritesheet("assets/spritesheet.png");
        MyGame {
            // sound: resources.load_sound("https://upload.wikimedia.org/wikipedia/commons/f/f2/Median_test.ogg"),
            sound: resources.load_sound("assets/blip.ogg"),
            playing: false,
            last_click: [0.0, 0.0],
            test: [
                spritesheet.create_sprite(0, 0, 11, 11),
                spritesheet.create_sprite(11, 0, 11, 11),
                spritesheet.create_sprite(0, 11, 11, 11),
            ],
        }
    }

    fn update(&mut self, mut context: GameUpdate) {
        for event in context.events() {
            if let MouseMove { x, y, buttons: _ } = event {
                self.last_click = [x as f64, y as f64];
            } else {
                log::info!("event: {:?}", event);
            }
        }

        let [x, y] = self.last_click;

        if x > 500.0 {
            if !self.playing {
                self.playing = true;
                self.sound.play();
            }
        } else {
            if self.playing {
                self.playing = false;
                self.sound.stop();
            }
        }

        self.test[0].draw(30.0, 10.0);
        self.test[2].draw(50.0, 10.0);
        self.test[1].draw(x - 5.0, y - 5.0);
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    panic::setup_panic_hook();

    info!("main function called");

    MyGame::run();
}
