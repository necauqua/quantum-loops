use wasm_bindgen::{prelude::*, *};
use web_sys::HtmlImageElement;

use crate::engine::surface::Surface;
use crate::engine::util::Mut;

#[derive(Clone)]
pub struct Spritesheet {
    surface: Mut<Surface>,
    image: Mut<Option<HtmlImageElement>>,
}

impl Spritesheet {
    pub(super) fn load(surface: Mut<Surface>, url: &str) -> Spritesheet {
        let element = HtmlImageElement::new().expect("Failed to create an Image instance");
        element
            .set_attribute("src", url)
            .expect("Failed to set img.src attribute");

        let image = if element.complete() {
            Mut::new(Some(element))
        } else {
            let image = Mut::new(None);
            let moved_image = image.clone();
            element
                .clone()
                .add_event_listener_with_callback(
                    "load",
                    Closure::once_into_js(move |_e: web_sys::Event| {
                        *moved_image.borrow_mut() = Some(element)
                    })
                    .unchecked_ref(),
                )
                .unwrap();
            image
        };
        Spritesheet { surface, image }
    }

    pub fn create_sprite(&self, u: u32, v: u32, w: u32, h: u32) -> Sprite {
        Sprite {
            parent: self.clone(),
            u,
            v,
            w,
            h,
            scale: 1.0,
        }
    }
}

pub struct Sprite {
    parent: Spritesheet,
    u: u32,
    v: u32,
    w: u32,
    h: u32,
    scale: f64,
}

impl Sprite {
    pub fn draw(&self, x: f64, y: f64) {
        if let Some(ref image) = *self.parent.image.borrow() {
            self.parent
                .surface
                .borrow()
                .context()
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    image,
                    self.u as f64,
                    self.v as f64,
                    self.w as f64,
                    self.h as f64,
                    x,
                    y,
                    (self.w as f64) * self.scale,
                    (self.h as f64) * self.scale,
                )
                .expect("Failed to draw sprite image");
        }
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
}
