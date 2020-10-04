use std::{
    cell::RefCell,
    collections::VecDeque,
    rc::Rc,
};
use nalgebra::Point2;
use wasm_bindgen::{*, prelude::*};
use web_sys::{AudioContext, CanvasRenderingContext2d, Document, HtmlCanvasElement, HtmlElement, Window};

use event::Event;
use sound::{Music, Sound};
use sprite::Spritesheet;

pub mod event;
pub mod sprite;
pub mod sound;
pub mod util;

pub fn window() -> Window {
    web_sys::window().expect("No window")
}

pub fn document() -> Document {
    window().document().expect("No document")
}

pub fn body() -> HtmlElement {
    document().body().expect("No document.body")
}

pub fn time() -> f64 {
    js_sys::Date::now() / 1e3
}

fn setup_canvas(event_queue: Rc<RefCell<VecDeque<Event>>>) -> CanvasRenderingContext2d {
    let canvas = document().create_element("canvas")
        .map_err(|_| ())
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().map_err(|_| ()))
        .expect("Failed to create canvas");

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .ok().flatten()
        .and_then(|obj| obj.dyn_into::<CanvasRenderingContext2d>().ok())
        .expect("No canvas 2d context?");

    fn update(canvas: &HtmlCanvasElement, context: &CanvasRenderingContext2d, window: &Window) {
        let ratio = window.device_pixel_ratio();
        let width = window.inner_width().unwrap().as_f64().unwrap();
        let height = window.inner_height().unwrap().as_f64().unwrap();
        canvas.set_width((width * ratio) as u32);
        canvas.set_height((height * ratio) as u32);

        let style = format!("width: {}px; height: {}px;", width, height);
        canvas.set_attribute("style", &style).unwrap();

        context.scale(ratio, ratio).unwrap();
    }

    let moved_window = window();
    let moved_canvas = canvas.clone();
    let moved_context = context.clone();
    let on_resize = Closure::wrap(Box::new(move |_e| {
        update(&moved_canvas, &moved_context, &moved_window);
    }) as Box<dyn FnMut(web_sys::Event)>);

    let w = window();
    w.add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .unwrap();
    on_resize.forget();

    update(&canvas, &context, &w);

    body().append_child(&canvas).expect("Failed to add canvas");

    event::setup_events(&canvas, event_queue);

    context
}

#[derive(Clone)]
pub struct Events(Rc<RefCell<VecDeque<Event>>>);

impl Iterator for Events {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.borrow_mut().pop_back()
    }
}

pub struct GameUpdate<'a, G: Game> {
    game: &'a mut G,
    delta_time: f64,
    size: Point2<f32>,
    surface: &'a CanvasRenderingContext2d,
    audio_context: &'a AudioContext,
}

pub enum StateTransition<G> {
    None,
    Set(Box<dyn GameState<G>>),
    Push(Box<dyn GameState<G>>),
    Pop,
}

impl<'a, G: Game> GameUpdate<'a, G> {
    pub fn game(&self) -> &G {
        self.game
    }

    pub fn game_mut(&mut self) -> &mut G {
        self.game
    }

    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }

    pub fn size(&self) -> Point2<f32> {
        self.size
    }

    pub fn surface(&self) -> &CanvasRenderingContext2d {
        self.surface
    }

    pub fn audio_context(&self) -> &AudioContext {
        self.audio_context
    }
}

fn run<G: Game + 'static>() {
    let event_queue = Rc::new(RefCell::new(VecDeque::new()));

    let surface = setup_canvas(event_queue.clone());
    let audio_context = AudioContext::new().unwrap();
    let canvas = surface.canvas().unwrap();

    let mut state_stack = VecDeque::new();

    let (mut game, current_state) = G::load(Resources {
        surface: surface.clone(),
        audio_context: audio_context.clone(),
    });
    state_stack.push_front(current_state);
    state_stack[0].on_mounted(&mut GameUpdate {
        game: &mut game,
        delta_time: 0.0,
        size: [canvas.width() as f32, canvas.height() as f32].into(),
        surface: &surface,
        audio_context: &audio_context,
    });

    let mut last_time = time();

    fn request_animation_frame(window: &Window, f: &Closure<dyn FnMut()>) {
        window
            .request_animation_frame(f.as_ref().unchecked_ref())
            .unwrap();
    }

    let window_moved = window();

    let rc1 = Rc::new(RefCell::new(None));
    let rc2 = rc1.clone();

    *rc1.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        surface.set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0).unwrap();
        let width = canvas.width() as f32;
        let height = canvas.height() as f32;

        let now = time();

        let mut context = GameUpdate {
            game: &mut game,
            delta_time: now - last_time,
            size: [width, height].into(),
            surface: &surface,
            audio_context: &audio_context,
        };

        let top_state = &mut state_stack[0];

        let transition = loop {
            if let Some(event) = event_queue.borrow_mut().pop_back() {
                match top_state.on_event(event, &mut context) {
                    StateTransition::None => (),
                    x => break x,
                }
            } else {
                break top_state.update(&mut context);
            }
        };

        match transition {
            StateTransition::Set(state) => {
                *top_state = state;
                top_state.on_mounted(&mut context);
            }
            StateTransition::Push(state) => {
                state_stack.push_front(state);
                state_stack[0].on_mounted(&mut context);
            }
            StateTransition::Pop => {
                if state_stack.len() == 1 {
                    panic!("Trying to pop last state!!");
                }
                state_stack.pop_front();
            }
            StateTransition::None => {}
        }

        last_time = now;

        request_animation_frame(&window_moved, rc2.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(&window(), rc1.borrow().as_ref().unwrap());
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

// copying Amethyst so hard accidentaly
// well their state design is pretty good I guess
pub trait GameState<G: Game> where Self: 'static {
    fn on_mounted(&mut self, _context: &mut GameUpdate<G>) {}

    fn on_event(&mut self, _event: Event, _context: &mut GameUpdate<G>) -> StateTransition<G> {
        StateTransition::None
    }

    fn update(&mut self, _context: &mut GameUpdate<G>) -> StateTransition<G> {
        StateTransition::None
    }
}

pub trait Game where Self: Sized + 'static {
    fn load(resources: Resources) -> (Self, Box<dyn GameState<Self>>);
}

pub trait GameRun: Game + Sized + private::Sealed {
    fn run() {
        run::<Self>()
    }
}

mod private {
    pub trait Sealed {}
}

impl<G: Game + Sized> private::Sealed for G {}

impl<G: Game + Sized + private::Sealed> GameRun for G {}
