use std::{
    cell::RefCell,
    panic::{set_hook, take_hook},
    rc::Rc
};

use wasm_bindgen::{*, prelude::*};
use web_sys::console::error_1;

pub fn setup_panic_hook() {
    let default_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        let msg = panic_info.to_string().into();
        let _ = js_sys::Reflect::set(js_sys::global().as_ref(), &"$_GAME_ERROR".into(), &msg);
        error_1(&msg);
        default_hook(panic_info);
    }));
}

pub trait PromiseGlue {
    fn rust_then<R: JsCast>(&self, f: impl FnMut(R) + 'static) -> js_sys::Promise;
}

impl PromiseGlue for js_sys::Promise {
    fn rust_then<R: JsCast>(&self, mut f: impl FnMut(R) + 'static) -> js_sys::Promise {
        let cb = Rc::new(RefCell::new(None));
        let other = cb.clone();
        *cb.borrow_mut() = Some(Closure::wrap(Box::new(move |res: JsValue| {
            f(res.dyn_into().unwrap());
            let _ = other.borrow_mut().take();
        }) as Box<dyn FnMut(JsValue)>));
        let borrowed = cb.borrow();

        self.then(borrowed.as_ref().unwrap())
    }
}

#[derive(Debug)]
pub struct SmoothChange {
    value: f32,
    prev_value: f32,
    next_value: f32,
    speed: f32,
}

impl SmoothChange {

    pub fn new(value: f32, speed: f32) -> Self {
        Self { value, prev_value: value, next_value: value, speed }
    }

    pub fn get(&self) -> f32 {
        self.next_value
    }

    pub fn set(&mut self, value: f32) {
        self.next_value = value;
    }

    pub fn get_interp(&self) -> f32 {
        self.value
    }

    pub fn update(&mut self, delta_time: f64) {

        // ZERO CLUE HOW THIS WORKS BUT OKAY

        let dist = self.next_value - self.prev_value;
        if dist.abs() <= f32::EPSILON {
            return
        }

        self.value += dist * delta_time as f32 * self.speed;

        if self.value - self.next_value <= f32::EPSILON {
            self.prev_value = self.next_value;
        }
    }
}

lazy_static::lazy_static! {
    static ref REM_TO_PIXEL_RATIO: f32 = {
        let window = web_sys::window().unwrap();
        window.get_computed_style(&window.document().unwrap().document_element().unwrap())
            .unwrap() // yey unwraps
            .unwrap()
            .get_property_value("font-size")
            .unwrap()
            .strip_suffix("px").unwrap_or("12")
            .parse::<f32>()
            .unwrap() * window.device_pixel_ratio() as f32
    };
}

pub trait RemConversions: Copy {

    fn rem_to_pixels(self) -> Self;

    fn to_rem(self) -> Self;
}

impl RemConversions for f32 {
    fn rem_to_pixels(self) -> Self {
        self * *REM_TO_PIXEL_RATIO
    }

    fn to_rem(self) -> Self {
        self / *REM_TO_PIXEL_RATIO
    }
}

impl RemConversions for f64 {
    fn rem_to_pixels(self) -> Self {
        self * *REM_TO_PIXEL_RATIO as f64
    }

    fn to_rem(self) -> Self {
        self / *REM_TO_PIXEL_RATIO as f64
    }
}
