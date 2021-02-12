use nalgebra::Vector2;
use wasm_bindgen::{prelude::*, *};
use web_sys::{EventTarget, MouseEvent, TouchEvent, WheelEvent};

use crate::engine::util::Mut;

pub trait ListenForever {
    fn listen_forever<E: JsCast>(&self, event_type: &str, f: impl FnMut(E) + 'static);
}

impl ListenForever for EventTarget {
    fn listen_forever<E: JsCast>(&self, event_type: &str, mut f: impl FnMut(E) + 'static) {
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| f(e.dyn_into().unwrap()))
            as Box<dyn FnMut(web_sys::Event)>);

        self.add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }
}

pub(super) fn setup_keyboard_events(target: &EventTarget, events: Mut<Vec<Event>>) {
    fn get_meta(e: web_sys::KeyboardEvent) -> KeyMeta {
        KeyMeta {
            repeat: e.repeat(),
            alt: e.alt_key(),
            shift: e.shift_key(),
            ctrl: e.ctrl_key(),
            meta: e.meta_key(),
        }
    }

    let moved_events = events.clone();
    target.listen_forever("keydown", move |e: web_sys::KeyboardEvent| {
        moved_events.borrow_mut().push(Event::KeyDown {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });

    let moved_events = events; //.clone();
    target.listen_forever("keyup", move |e: web_sys::KeyboardEvent| {
        moved_events.borrow_mut().push(Event::KeyUp {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });
}

pub(super) fn setup_touch_events(target: &EventTarget, events: Mut<Vec<Event>>) {
    target.listen_forever("contextmenu", |e: web_sys::Event| e.prevent_default());

    let ratio = super::window().device_pixel_ratio();

    let moved_event_queue = events.clone();
    target.listen_forever("mouseup", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseUp {
            pos: [(e.client_x() as f64 * ratio), (e.client_y() as f64 * ratio)].into(),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("mousedown", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseDown {
            pos: [(e.client_x() as f64 * ratio), (e.client_y() as f64 * ratio)].into(),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("mousemove", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseMove {
            pos: [(e.client_x() as f64 * ratio), (e.client_y() as f64 * ratio)].into(),
            buttons: MouseButton::from_bitmap(e.buttons()),
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("wheel", move |e: WheelEvent| {
        moved_event_queue.borrow_mut().push(Event::MouseWheel {
            pos: [(e.client_x() as f64 * ratio), (e.client_y() as f64 * ratio)].into(),
            delta: [e.delta_x(), e.delta_y()].into(),
            buttons: MouseButton::from_bitmap(e.buttons()),
        });
    });

    fn get_touches(e: TouchEvent, ratio: f64) -> Box<[Vector2<f64>]> {
        let touch_list = e.touches();
        let mut touches = Vec::with_capacity(touch_list.length() as usize);
        while let Some(t) = touch_list.get(touches.len() as u32) {
            touches.push([t.client_x() as f64 * ratio, t.client_y() as f64 * ratio].into());
        }
        touches.into_boxed_slice()
    }

    let moved_event_queue = events.clone();
    target.listen_forever("touchstart", move |e: TouchEvent| {
        // prevent mouse emulation if any
        e.prevent_default();
        moved_event_queue.borrow_mut().push(Event::TouchStart {
            touches: get_touches(e, ratio),
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("touchmove", move |e: TouchEvent| {
        e.prevent_default();
        moved_event_queue.borrow_mut().push(Event::TouchMove {
            touches: get_touches(e, ratio),
        });
    });

    let moved_event_queue = events.clone();
    target.listen_forever("touchend", move |e: TouchEvent| {
        e.prevent_default();
        moved_event_queue.borrow_mut().push(Event::TouchEnd {
            touches: get_touches(e, ratio),
        });
    });
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct KeyMeta {
    repeat: bool,
    alt: bool,
    shift: bool,
    ctrl: bool,
    meta: bool,
}

#[derive(Debug, Clone)]
pub enum Event {
    MouseDown {
        pos: Vector2<f64>,
        button: MouseButton,
    },
    MouseUp {
        pos: Vector2<f64>,
        button: MouseButton,
    },
    MouseMove {
        pos: Vector2<f64>,
        buttons: Vec<MouseButton>,
    },
    MouseWheel {
        pos: Vector2<f64>,
        buttons: Vec<MouseButton>,
        delta: Vector2<f64>,
    },
    TouchStart {
        touches: Box<[Vector2<f64>]>,
    },
    TouchMove {
        touches: Box<[Vector2<f64>]>,
    },
    TouchEnd {
        touches: Box<[Vector2<f64>]>,
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
