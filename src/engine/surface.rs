use nalgebra::Vector2;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::engine::event::Event;
use crate::engine::util::Mut;

#[derive(Clone)]
pub struct Surface {
    size: Mut<Vector2<f64>>,
    context: CanvasRenderingContext2d,
}

fn setup_canvas(events: Mut<Vec<Event>>, size: Mut<Vector2<f64>>) -> CanvasRenderingContext2d {
    let canvas = super::document()
        .create_element("canvas")
        .map_err(|_| ())
        .and_then(|e| e.dyn_into::<HtmlCanvasElement>().map_err(|_| ()))
        .expect("Failed to create canvas");

    let context: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|obj| obj.dyn_into::<CanvasRenderingContext2d>().ok())
        .expect("No canvas 2d context?");

    let moved_window = super::window();
    let moved_canvas = canvas.clone();
    let moved_context = context.clone();
    let moved_size = size.clone();
    let resize = move || {
        let ratio = moved_window.device_pixel_ratio();

        let width = moved_window
            .inner_width()
            .ok()
            .and_then(|js| js.as_f64())
            .unwrap();
        let height = moved_window
            .inner_height()
            .ok()
            .and_then(|js| js.as_f64())
            .unwrap();

        let scaled_width = width * ratio;
        let scaled_height = height * ratio;

        moved_canvas.set_width(scaled_width as u32);
        moved_canvas.set_height(scaled_height as u32);

        let style = format!("width: {}px; height: {}px;", width, height);
        moved_canvas.set_attribute("style", &style).unwrap();

        moved_context.scale(ratio, ratio).unwrap();

        moved_context.set_text_align("center");
        moved_context.set_text_baseline("middle");

        *moved_size.borrow_mut() = [scaled_width, scaled_height].into();
    };
    resize();

    let on_resize = Closure::wrap(Box::new(move |_e| resize()) as Box<dyn FnMut(web_sys::Event)>);

    super::window()
        .add_event_listener_with_callback("resize", on_resize.as_ref().unchecked_ref())
        .unwrap();

    on_resize.forget();

    super::body()
        .append_child(&canvas)
        .expect("Failed to add canvas");

    super::event::setup_touch_events(&canvas, events.clone());
    super::event::setup_keyboard_events(&super::document(), events);

    context
}

impl Surface {
    pub fn new(events: Mut<Vec<Event>>) -> Self {
        let size = Mut::new([0.0, 0.0].into());
        let context = setup_canvas(events, size.clone());
        Self { size, context }
    }

    pub fn context(&self) -> CanvasRenderingContext2d {
        self.context.clone()
    }

    pub fn size(&self) -> Vector2<f64> {
        *self.size.borrow()
    }
}
