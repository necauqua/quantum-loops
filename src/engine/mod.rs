use std::{
    cell::RefCell,
    collections::{
        vec_deque::Drain,
        VecDeque,
    },
    rc::Rc,
};

use wasm_bindgen::{*, prelude::*};
use web_sys::{CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, Window, AudioContext};

use event::Event;

use crate::engine::sprite::Spritesheet;
use crate::engine::sound::{Music, Sound};

pub mod event;
pub mod sprite;
pub mod sound;
pub mod util;

fn window() -> Window {
    web_sys::window().expect("No window")
}

fn document() -> Document {
    window().document().expect("No document")
}

fn body() -> HtmlElement {
    document().body().expect("No document.body")
}

fn setup_canvas(event_queue: Rc<RefCell<VecDeque<Event>>>) -> CanvasRenderingContext2d {
    let canvas = document().create_element("canvas")
        .map_err(|_| ())
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().map_err(|_| ()))
        .expect("Failed to create canvas");

    let w = window();
    let c = canvas.clone();
    let on_resize = Closure::wrap(Box::new(move |_e| {
        c.set_width(w.inner_width().unwrap().as_f64().unwrap() as u32);
        c.set_height(w.inner_height().unwrap().as_f64().unwrap() as u32);
    }) as Box<dyn FnMut(web_sys::Event)>);

    let w = window();
    w.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .unwrap();
    on_resize.forget();

    canvas.set_width(w.inner_width().unwrap().as_f64().unwrap() as u32);
    canvas.set_height(w.inner_height().unwrap().as_f64().unwrap() as u32);

    body().append_child(&canvas).expect("Failed to add canvas");

    event::setup_events(&canvas, event_queue);

    canvas
        .get_context("2d")
        .ok().flatten()
        .and_then(|obj| obj.dyn_into::<CanvasRenderingContext2d>().ok())
        .expect("No canvas 2d context?")
}

pub struct GameUpdate<'a> {
    dt: f64,
    surface: &'a CanvasRenderingContext2d,
    audio_context: &'a AudioContext,
    events: Drain<'a, Event>,
}

impl<'a> GameUpdate<'a> {
    pub fn delta_time(&self) -> f64 {
        self.dt
    }

    pub fn events(&mut self) -> &mut Drain<'a, Event> {
        &mut self.events
    }
}

fn run<G: Game + 'static>() {
    let event_queue = Rc::new(RefCell::new(VecDeque::new()));

    let surface = setup_canvas(event_queue.clone());
    let audio_context = AudioContext::new().unwrap();

    let mut game = G::load(Resources {
        surface: surface.clone(),
        audio_context: audio_context.clone(),
    });

    let mut last_time: f64 = js_sys::Date::now();

    fn request_animation_frame(window: &Window, f: &Closure<dyn FnMut()>) {
        window
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }

    let window_moved = window();

    let cb = Rc::new(RefCell::new(None));
    let outer = cb.clone();

    *outer.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let canvas = surface.canvas().unwrap();
        surface.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        surface.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);

        let now = js_sys::Date::now();
        game.update(GameUpdate {
            dt: (now - last_time) / 1e3,
            surface: &surface,
            audio_context: &audio_context,
            events: event_queue.borrow_mut().drain(..),
        });
        last_time = now;

        request_animation_frame(&window_moved, cb.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(&window(), outer.borrow().as_ref().unwrap());
}

pub struct Resources {
    surface: CanvasRenderingContext2d,
    audio_context: AudioContext,
}

impl Resources {
    pub fn load_spritesheet(&self, url: &str) -> Spritesheet {
        Spritesheet::load(self.surface.clone(), url)
    }

    pub fn load_sound(&self, url: &str) -> Sound {
        Sound::load(self.audio_context.clone(), url)
    }

    pub fn load_music(&self, url: &str) -> Music {
        Music::load(url)
    }
}

pub trait Game {
    fn load(resources: Resources) -> Self;

    fn update(&mut self, context: GameUpdate);
}

mod private {
    pub trait Sealed {}
}

pub trait RunGame: Game + Sized + 'static + private::Sealed {
    fn run() {
        run::<Self>()
    }
}

impl<T: Game + Sized + 'static> private::Sealed for T {}

impl<T: Game + Sized + 'static + private::Sealed> RunGame for T {}
