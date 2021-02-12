#![allow(dead_code)]

use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use wasm_bindgen::{prelude::*, *};
use web_sys::{Document, HtmlElement, Window};

use event::Event;
use sound::{Sound, SoundContext};
use sprite::Spritesheet;
use util::Mut;

use crate::engine::surface::Surface;
use std::cell::{Ref, RefMut};

pub mod event;
pub mod sound;
pub mod sprite;
pub mod surface;
pub mod ui;
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

fn get_data<D: Default + for<'a> Deserialize<'a>>() -> D {
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get("data")
        .unwrap()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn set_data<D: Serialize>(data: &D) {
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .set("data", &serde_json::to_string(data).unwrap())
        .unwrap()
}

fn compute_rem_to_pixel_ratio() -> f64 {
    let window = window();
    window
        .get_computed_style(&document().document_element().unwrap())
        .ok()
        .flatten()
        .and_then(|style| style.get_property_value("font-size").ok())
        .as_deref()
        .unwrap_or("")
        .strip_suffix("px")
        .unwrap_or("12")
        .parse()
        .unwrap_or(12.0)
        * window.device_pixel_ratio()
}

#[derive(Debug)]
pub enum StateTransition<G: Game> {
    None,
    Set(Box<dyn GameState<G>>),
    Push(Box<dyn GameState<G>>),
    Pop,
}

impl<G: Game> StateTransition<G> {
    pub fn is_none(&self) -> bool {
        match self {
            StateTransition::None => true,
            _ => false,
        }
    }

    #[inline]
    pub fn set<S: GameState<G>>(state: S) -> StateTransition<G> {
        StateTransition::Set(Box::new(state))
    }

    #[inline]
    pub fn push<S: GameState<G>>(state: S) -> StateTransition<G> {
        StateTransition::Push(Box::new(state))
    }
}

pub struct Context<'a, G: Game> {
    delta_time: f64,
    rem_to_px: f64,
    surface: Mut<Surface>,
    sound_context: Mut<SoundContext>,
    storage: &'a mut G::Storage,
    pub game: &'a mut G,
}

impl<'a, G: Game> Context<'a, G> {
    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }

    pub fn rem_to_px(&self, rem: f64) -> f64 {
        rem * self.rem_to_px
    }

    pub fn surface(&self) -> Ref<Surface> {
        self.surface.borrow()
    }

    pub fn sound_context_mut(&self) -> RefMut<SoundContext> {
        self.sound_context.borrow_mut()
    }

    pub fn storage(&self) -> &G::Storage {
        self.storage
    }

    pub fn set_storage(&mut self, new_storage: G::Storage) {
        set_data(&new_storage);
        *self.storage = new_storage;
    }
}

fn handle_transition<G: Game>(
    stack: &mut Vec<Box<dyn GameState<G>>>,
    mut trn: impl FnMut(&mut Box<dyn GameState<G>>, &mut Context<G>) -> StateTransition<G>,
    mut context: Context<G>,
) {
    let mut next_transition = Some(trn(stack.last_mut().unwrap(), &mut context));

    while let Some(transition) = next_transition.take() {
        match transition {
            StateTransition::Set(state) => {
                let last = stack.last_mut().unwrap();
                *last = state;
                next_transition = Some(last.on_pushed(&mut context));
            }
            StateTransition::Push(state) => {
                stack.push(state);
                next_transition = Some(stack.last_mut().unwrap().on_pushed(&mut context));
            }
            StateTransition::Pop => {
                let next = stack.pop().unwrap().on_popped(&mut context);
                if let StateTransition::Push(_) = next {
                    // noop
                } else if stack.is_empty() {
                    panic!("Popped the last state!");
                }
                next_transition = Some(next);
            }
            StateTransition::None => {}
        }
    }
}

fn run<G: Game>() {
    let event_queue = Mut::new(Vec::new());

    let surface = Mut::new(Surface::new(event_queue.clone()));
    let sound_context = Mut::new(SoundContext::new());

    let (mut game, current_state) = G::load(Resources {
        surface: surface.clone(),
        sound_context: sound_context.clone(),
    });
    let mut storage = get_data();

    let mut states = vec![current_state];
    handle_transition(
        &mut states,
        |state, context| state.on_pushed(context),
        Context {
            delta_time: 0.0,
            rem_to_px: compute_rem_to_pixel_ratio(),
            surface: surface.clone(),
            sound_context: sound_context.clone(),
            game: &mut game,
            storage: &mut storage,
        },
    );

    let mut last_time = time();

    let window_moved = window();

    let rc1: Mut<Option<Closure<dyn FnMut()>>> = Mut::new(None);
    //       ^ well, Rust failed to get that type somehow due to request_animation_frame call
    let rc2 = rc1.clone();

    *rc1.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        surface
            .borrow()
            .context()
            .set_transform(1.0, 0.0, 0.0, 1.0, 0.0, 0.0)
            .unwrap();

        let now = time();

        handle_transition(
            &mut states,
            |state, context| loop {
                if let Some(event) = event_queue.borrow_mut().pop() {
                    match state.on_event(event, context) {
                        StateTransition::None => (),
                        x => break x,
                    }
                } else {
                    break state.on_update(context);
                }
            },
            Context {
                delta_time: now - last_time,
                rem_to_px: compute_rem_to_pixel_ratio(),
                surface: surface.clone(),
                sound_context: sound_context.clone(),
                game: &mut game,
                storage: &mut storage,
            },
        );

        last_time = now;

        window_moved
            .request_animation_frame(rc2.borrow().as_ref().unwrap().as_ref().unchecked_ref())
            .unwrap();
    }) as Box<dyn FnMut()>));

    window()
        .request_animation_frame(rc1.borrow().as_ref().unwrap().as_ref().unchecked_ref())
        .unwrap();
}

pub struct Resources {
    surface: Mut<Surface>,
    sound_context: Mut<SoundContext>,
}

impl Resources {
    pub fn load_spritesheet(&self, url: &str) -> Spritesheet {
        Spritesheet::load(self.surface.clone(), url)
    }

    pub fn load_sound(&self, url: &str) -> Sound {
        Sound::load(self.sound_context.clone(), url)
    }
}

// copying Amethyst so hard accidentaly
// well their state design is pretty good I guess
pub trait GameState<G: Game>
where
    Self: Debug + 'static,
{
    fn on_pushed(&mut self, _context: &mut Context<G>) -> StateTransition<G> {
        StateTransition::None
    }

    fn on_event(&mut self, _event: Event, _context: &mut Context<G>) -> StateTransition<G> {
        StateTransition::None
    }

    fn on_update(&mut self, _context: &mut Context<G>) -> StateTransition<G> {
        StateTransition::None
    }

    fn on_popped(self: Box<Self>, _context: &mut Context<G>) -> StateTransition<G> {
        StateTransition::None
    }
}

pub trait Game
where
    Self: Debug + Sized + 'static,
{
    type Storage: Clone + Default + Serialize + for<'a> Deserialize<'a>;

    fn load(resources: Resources) -> (Self, Box<dyn GameState<Self>>);
}

pub trait GameRun: Game + private::Sealed {
    fn run() {
        run::<Self>()
    }
}

mod private {
    pub trait Sealed {}
}

impl<G: Game> private::Sealed for G {}

impl<G: Game + private::Sealed> GameRun for G {}
