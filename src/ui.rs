use std::borrow::Cow;

use nalgebra::Point2;
use web_sys::CanvasRenderingContext2d;

use crate::{engine::{
    event::{Event, MouseButton},
    Context,
    util::RemConversions,
}, QuantumLoops, states::
            main_game::{DISABLED_TEXT_COLOR, HOVERED_TEXT_COLOR, TEXT_COLOR}, sounds_enabled};

#[derive(Debug)]
pub struct Button {
    pub pos: Point2<f32>,
    pub text: Cow<'static, str>,
    size: f32,
    font: String,
    hovered: bool,
    pub enabled: bool,
}

impl Button {
    pub fn new(text: Cow<'static, str>) -> Self {
        Self {
            pos: [0.0, 0.0].into(),
            text,
            size: 2.5,
            font: "2.5rem monospace".into(),
            hovered: false,
            enabled: true,
        }
    }

    pub fn with_pos(mut self, pos: Point2<f32>) -> Self {
        self.pos = pos;
        self
    }

    pub fn with_size(mut self, size: f32) -> Self {
        self.size = size;
        self.font = format!("{}rem monospace", size);
        self
    }

    pub fn compute_size(&self, surface: &CanvasRenderingContext2d) -> (f32, f32) {
        surface.set_font(&self.font);
        let dim = surface.measure_text(&self.text).unwrap();
        (dim.width() as f32, self.size.rem_to_pixels())
    }

    pub fn is_over(&self, pos: Point2<f32>, surface: &CanvasRenderingContext2d) -> bool {
        let (w, h) = self.compute_size(surface);
        pos.x >= self.pos.x - w / 2.0 && pos.x <= self.pos.x + w / 2.0
            && pos.y >= self.pos.y - h / 2.0 && pos.y <= self.pos.y + h / 2.0
    }

    pub fn on_event(&mut self, event: &Event, context: &mut Context<QuantumLoops>) -> bool {
        if !self.enabled {
            return false;
        }
        match event {
            Event::MouseMove { pos, .. } => {
                let over = self.is_over(*pos, context.surface());
                if !self.hovered && over && sounds_enabled() {
                    context.game().sounds.hover.play();
                }
                self.hovered = over;
            }
            Event::MouseUp { pos, button: MouseButton::Left } => {
                if self.is_over(*pos, context.surface()) {
                    if sounds_enabled() {
                        context.game().sounds.click.play();
                    }
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    pub fn render(&mut self, context: &mut Context<QuantumLoops>) {
        let surface = context.surface();

        let color = if !self.enabled {
            DISABLED_TEXT_COLOR
        } else if self.hovered {
            HOVERED_TEXT_COLOR
        } else {
            TEXT_COLOR
        };

        surface.set_fill_style(&color.into());

        let px = self.pos.x as f64;
        let py = self.pos.y as f64;

        surface.set_font(&self.font);
        surface.fill_text(&self.text, px, py).unwrap();
    }
}
