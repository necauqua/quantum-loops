use web_sys::{HtmlCanvasElement, MouseEvent, EventTarget};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::VecDeque;
use wasm_bindgen::{*, prelude::*};

trait EventListen {

    fn listen<E: JsCast>(&self, event_type: &str, f: impl FnMut(E) + 'static);
}

impl<T: AsRef<EventTarget>> EventListen for T {
    fn listen<E: JsCast>(&self, event_type: &str, mut f: impl FnMut(E) + 'static)  {
        let closure = Closure::wrap(Box::new(move |e: web_sys::Event| {
            e.prevent_default();
            f(e.dyn_into().unwrap());
        }) as Box<dyn FnMut(web_sys::Event)>);
        self.as_ref().add_event_listener_with_callback(event_type, closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    }
}

pub(super) fn setup_events(canvas: &HtmlCanvasElement, event_queue: Rc<RefCell<VecDeque<Event>>>) {
    let on_contextmenu =
        Closure::wrap(Box::new(move |e: web_sys::Event| {
            e.prevent_default();
            JsValue::FALSE
        }) as Box<dyn FnMut(web_sys::Event) -> JsValue>);
    canvas.add_event_listener_with_callback("contextmenu", on_contextmenu.as_ref().unchecked_ref())
        .unwrap();
    on_contextmenu.forget();

    let moved_event_queue = event_queue.clone();
    canvas.listen("mouseup", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseUp {
            x: e.client_x(),
            y: e.client_y(),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = event_queue.clone();
    canvas.listen("mousedown", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseDown {
            x: e.client_x(),
            y: e.client_y(),
            button: match MouseButton::from_code(e.button()) {
                Some(b) => b,
                _ => return,
            },
        });
    });

    let moved_event_queue = event_queue.clone();
    canvas.listen("mousemove", move |e: MouseEvent| {
        moved_event_queue.borrow_mut().push_back(Event::MouseMove {
            x: e.client_x(),
            y: e.client_y(),
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
    document.listen("keydown", move |e: web_sys::KeyboardEvent| {
        moved_event_queue.borrow_mut().push_back(Event::KeyDown {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });

    let moved_event_queue = event_queue;//.clone();
    document.listen("keyup", move |e: web_sys::KeyboardEvent| {
        moved_event_queue.borrow_mut().push_back(Event::KeyUp {
            code: e.key_code(),
            key: e.key(),
            meta: get_meta(e),
        })
    });
}

#[derive(Debug)]
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
    MouseDown { x: i32, y: i32, button: MouseButton },
    MouseMove { x: i32, y: i32, buttons: Vec<MouseButton> },
    MouseUp { x: i32, y: i32, button: MouseButton },
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
