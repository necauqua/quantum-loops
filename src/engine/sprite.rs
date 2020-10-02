use std::{
    rc::Rc,
    cell::RefCell,
    sync::atomic::{AtomicBool, Ordering}
};

use wasm_bindgen::{*, prelude::*};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlImageElement};

#[derive(Clone)]
pub struct Spritesheet {
    surface: CanvasRenderingContext2d,
    image: HtmlImageElement,
    loaded: Rc<AtomicBool>,
}

impl Spritesheet {
    pub(super) fn load(surface: CanvasRenderingContext2d, url: &str) -> Spritesheet {
        let image = HtmlImageElement::new()
            .expect("Failed to create an Image instance");
        image.set_attribute("src", url).expect("Failed to set img.src attribute");

        let complete = image.complete();
        let loaded = Rc::new(AtomicBool::new(complete));

        if !complete {
            let cb = Rc::new(RefCell::new(None));
            let selfref = cb.clone();

            let loaded_ref = loaded.clone();

            *cb.borrow_mut() = Some(Closure::wrap(Box::new(move |_e| {
                loaded_ref.store(true, Ordering::Relaxed);
                // free the closure
                let _ = selfref.borrow_mut().take();
            }) as Box<dyn Fn(web_sys::Event)>));

            image.add_event_listener_with_callback("load", cb.borrow().as_ref().unwrap().as_ref().unchecked_ref()).unwrap();
        }
        Spritesheet {
            surface,
            image,
            loaded,
        }
    }
    
    pub fn create_sprite(&self, u: u32, v: u32, w: u32, h: u32) -> Sprite {
        Sprite {
            parent: self.clone(),
            u,
            v,
            w,
            h,
            scale: 1.0,
            tinted: None,
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
    tinted: Option<HtmlCanvasElement>,
}

impl Sprite {
    pub fn draw(&self, x: f64, y: f64) {
        if !self.parent.loaded.load(Ordering::Relaxed) {
            return;
        }
        if let Some(tinted) = self.tinted.as_ref() {
            self.parent.surface
                .draw_image_with_html_canvas_element_and_dw_and_dh(
                    tinted,
                    x, y,
                    (self.w as f64) * self.scale, (self.h as f64) * self.scale,
                )
                .expect("Failed to draw tinted sprite image")
        } else {
            self.parent.surface
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &self.parent.image,
                    self.u as f64, self.v as f64,
                    self.w as f64, self.h as f64,
                    x, y,
                    (self.w as f64) * self.scale, (self.h as f64) * self.scale,
                )
                .expect("Failed to draw sprite image");
        }
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn tinted(mut self, color: [u8; 3]) -> Self {
        let canvas = super::document().create_element("canvas")
            .ok()
            .and_then(|e| e.dyn_into::<HtmlCanvasElement>().ok())
            .expect("Failed to create tint buffer canvas");

        canvas.set_width(self.w);
        canvas.set_height(self.h);

        let surface: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .ok().flatten()
            .and_then(|obj| obj.dyn_into::<CanvasRenderingContext2d>().ok())
            .unwrap();

        fn draw_image(s: &Sprite, surface: &CanvasRenderingContext2d) {
            surface
                .draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    &s.parent.image,
                    s.u as f64, s.v as f64,
                    s.w as f64, s.h as f64,
                    0.0, 0.0,
                    s.w as f64, s.h as f64,
                )
                .expect("Failed to draw sprite image into tint buffer canvas");
        }

        draw_image(&self, &surface);
        surface.set_global_composite_operation("multiply").unwrap();
        surface.set_fill_style(&format!("#{:X}{:X}{:X}", color[0], color[1], color[2]).into());
        // surface.fill_rect(0 as f64, 0 as f64, self.w as f64, self.h as f64);

        // fix masking issues, if any
        surface.set_global_alpha(0.5);
        surface.set_global_composite_operation("destination-in").unwrap();
        draw_image(&self, &surface);

        self.tinted = Some(canvas);

        self
    }
}
