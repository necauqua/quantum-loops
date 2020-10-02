use std::panic::{set_hook, take_hook};

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
