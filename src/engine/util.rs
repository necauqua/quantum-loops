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
