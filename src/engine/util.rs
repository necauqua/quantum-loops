use std::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    ops::{Deref, DerefMut},
    panic::{set_hook, take_hook},
    rc::Rc,
};

use wasm_bindgen::prelude::*;
use web_sys::console::error_1;

#[wasm_bindgen]
extern "C" {
    fn _game_error(text: &str);
}

pub fn setup_panic_hook() {
    let default_hook = take_hook();
    set_hook(Box::new(move |panic_info| {
        let msg = panic_info.to_string();
        _game_error(&msg);
        error_1(&msg.into());
        default_hook(panic_info);
    }));
}

pub struct Mut<T> {
    inner: Rc<RefCell<T>>,
}

impl<T> Clone for Mut<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Mut<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }
}

impl<T: Default> Default for Mut<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T: Debug> Debug for Mut<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl<T> Deref for Mut<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Mut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct Bitmap {
    bits: u32,
}

impl Bitmap {
    pub const fn new(init: u32) -> Self {
        Self { bits: init }
    }

    pub const fn empty() -> Self {
        Self { bits: 0 }
    }

    pub const fn full() -> Self {
        Self { bits: !0 }
    }

    #[inline]
    pub fn on(&mut self, bit: u8) {
        self.bits |= 1 << bit;
    }

    #[inline]
    pub fn off(&mut self, bit: u8) {
        self.bits &= !(1 << bit);
    }

    #[inline]
    pub fn set(&mut self, bit: u8, value: bool) {
        if value {
            self.on(bit)
        } else {
            self.off(bit)
        }
    }

    #[inline]
    pub const fn with_on(mut self, bit: u8) -> Self {
        self.bits |= 1 << bit;
        self
    }

    #[inline]
    pub const fn with_off(mut self, bit: u8) -> Self {
        self.bits &= !(1 << bit);
        self
    }

    #[inline]
    pub const fn with_set(self, bit: u8, value: bool) -> Self {
        if value {
            self.with_on(bit)
        } else {
            self.with_off(bit)
        }
    }

    #[inline]
    pub const fn get(&self, bit: u8) -> bool {
        self.bits & (1 << bit) != 0
    }

    #[inline]
    pub const fn intersects(&self, other: Bitmap) -> bool {
        self.bits & other.bits != 0
    }
}

impl Default for Bitmap {
    fn default() -> Self {
        Self::empty()
    }
}

impl Debug for Bitmap {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Bitmap({:032b})", self.bits)
    }
}

#[derive(Debug)]
pub struct SmoothChange {
    value: f64,
    prev_value: f64,
    next_value: f64,
    speed: f64,
}

impl SmoothChange {
    pub fn new(value: f64, speed: f64) -> Self {
        Self {
            value,
            prev_value: value,
            next_value: value,
            speed,
        }
    }

    pub fn get(&self) -> f64 {
        self.next_value
    }

    pub fn set(&mut self, value: f64) {
        self.next_value = value;
    }

    pub fn set_raw(&mut self, value: f64) {
        self.prev_value = value;
        self.value = value;
        self.next_value = value;
    }

    pub fn get_interp(&self) -> f64 {
        self.value
    }

    pub fn update(&mut self, delta_time: f64) {
        let dist = self.next_value - self.prev_value;
        if dist.abs() <= f64::EPSILON {
            return;
        }
        self.value += dist * self.speed * delta_time;
        if self.value - self.next_value <= f64::EPSILON {
            self.prev_value = self.next_value;
        }
    }
}
