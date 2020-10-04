use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::Deref;
use std::rc::Rc;

use nalgebra::{Point2, Vector2};
use wasm_bindgen::{*, prelude::*};
use web_sys::{EventTarget, HtmlCanvasElement, MouseEvent, WheelEvent};

trait ListenForever {
    fn listen_forever<E: JsCast>(&self, event_type: &str, f: impl FnMut(E) + 'static);
}

impl<T: Deref<Target=EventTarget>> ListenForever for T {
    fn listen_forever<E: JsCast>(&self, event_type: &str, mut f: impl FnMut(E) + 'static) {
        let closure =
            Closure::wrap(Box::new(move |e: web_sys::Event|
                f(e.dyn_into().unwrap())) as Box<dyn FnMut(web_sys::Event)>);

        self.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
}

pub(super) fn setup_events(canvas: &HtmlCanvasElement, event_queue: Rc<RefCell<VecDeque<Event>>>) {
    canvas.listen_forever("contextmenu", |e: web_sys::Event| e.prevent_default());

    let ratio = super::window().device_pixel_ratio() as f32;

    let moved_event_queue = event_queue.clone();
    canvas.listen_forever("mouseup", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseUp {
            pos: [(e.client_x() as f32 * ratio), (e.client_y() as f32 * ratio)].into(),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = event_queue.clone();
    canvas.listen_forever("mousedown", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseDown {
            pos: [(e.client_x() as f32 * ratio), (e.client_y() as f32 * ratio)].into(),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = event_queue.clone();
    canvas.listen_forever("mousemove", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseMove {
            pos: [(e.client_x() as f32 * ratio), (e.client_y() as f32 * ratio)].into(),
            buttons: MouseButton::from_bitmap(e.buttons()),
        });
    });

    let moved_event_queue = event_queue.clone();
    canvas.listen_forever("wheel", move |e: WheelEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseWheel {
            pos: [(e.client_x() as f32 * ratio), (e.client_y() as f32 * ratio)].into(),
            delta: [e.delta_x() as f32, e.delta_y() as f32].into(),
            buttons: MouseButton::from_bitmap(e.buttons()),
        });
    });

    fn get_meta(e: web_sys::KeyboardEvent) -> KeyMeta {
        KeyMeta {
            repeat: e.repeat(),
            alt: e.alt_key(),
            shift: e.shift_key(),
            ctrl: e.ctrl_key(),
            meta: e.meta_key(),
        }
    }

    let document = super::document();

    let moved_event_queue = event_queue.clone();
    document.listen_forever("keydown", move |e: web_sys::KeyboardEvent| {
        moved_event_queue.borrow_mut().push_back(Event::KeyDown {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });

    let moved_event_queue = event_queue;//.clone();
    document.listen_forever("keyup", move |e: web_sys::KeyboardEvent| {
        moved_event_queue.borrow_mut().push_back(Event::KeyUp {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });
}

#[derive(Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
}

impl MouseButton {
    pub fn from_code(code: i16) -> Option<MouseButton> {
        match code {
            0 => Some(MouseButton::Left),
            1 => Some(MouseButton::Middle),
            2 => Some(MouseButton::Right),
            3 => Some(MouseButton::Back),
            4 => Some(MouseButton::Forward),
            _ => None,
        }
    }

    pub fn from_bitmap(bits: u16) -> Vec<MouseButton> {
        let mut buttons = Vec::new();
        if bits & 1 != 0 {
            buttons.push(MouseButton::Left);
        }
        if bits & 2 != 0 {
            buttons.push(MouseButton::Right);
        }
        if bits & 4 != 0 {
            buttons.push(MouseButton::Middle);
        }
        if bits & 8 != 0 {
            buttons.push(MouseButton::Back);
        }
        if bits & 16 != 0 {
            buttons.push(MouseButton::Forward);
        }
        buttons
    }
}

#[derive(Debug)]
pub struct KeyMeta {
    repeat: bool,
    alt: bool,
    shift: bool,
    ctrl: bool,
    meta: bool,
}

#[derive(Debug)]
pub enum Event {
    MouseDown { pos: Point2<f32>, button: MouseButton },
    MouseUp { pos: Point2<f32>, button: MouseButton },
    MouseMove { pos: Point2<f32>, buttons: Vec<MouseButton> },
    MouseWheel {
        pos: Point2<f32>,
        buttons: Vec<MouseButton>,
        delta: Vector2<f32>,
    },
    KeyDown {
        code: u32,
        key: String,
        meta: KeyMeta,
    },
    KeyUp {
        code: u32,
        key: String,
        meta: KeyMeta,
    },
}
