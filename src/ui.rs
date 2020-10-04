use nalgebra::{Point2, Vector2};
use web_sys::CanvasRenderingContext2d;

use crate::{
    engine::{
        event::{Event, MouseButton},
        GameUpdate,
        util::RemConversions,
    },
    QuantumLoops,
    states::main_game::{HOVERED_TEXT_COLOR, TEXT_COLOR},
};

#[derive(Debug)]
pub struct Buttons {
    buttons: Vec<(Point2<f32>, String, bool)>,
}

fn is_over(surface: &CanvasRenderingContext2d, text: &str, pos: Point2<f32>, mouse: Point2<f32>) -> bool {
    let dim = surface.measure_text(&text).unwrap();
    let hw = (dim.width() / 2.0) as f32;
    let hh = 2.5.rem_to_pixels() / 2.0;
    mouse.x >= pos.x - hw && mouse.x <= pos.x + hw
        && mouse.y >= pos.y - hh && mouse.y <= pos.y + hh
}

impl Buttons {
    pub fn new() -> Self {
        Self {
            buttons: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.buttons.len()
    }

    pub fn add_button(&mut self, pos: Point2<f32>, text: String) {
        self.buttons.push((pos, text, false));
    }

    pub fn move_buttons(&mut self, offset: Vector2<f32>) {
        for (pos, _, _) in &mut self.buttons {
            *pos += offset;
        }
    }

    pub fn on_event(&mut self, event: &Event, context: &mut GameUpdate<QuantumLoops>) -> Option<usize> {
        let surface = context.surface();
        match event {
            Event::MouseMove { pos: mouse, .. } => {
                for (pos, text, hovered) in &mut self.buttons {
                    let over = is_over(surface, text, *pos, *mouse);
                    if !*hovered && over {
                        context.game().sounds.hover.play();
                    }
                    *hovered = over;
                }
            }
            Event::MouseUp { pos: mouse, button: MouseButton::Left } => {
                context.game().sounds.background.play();

                for (idx, (pos, text, _)) in self.buttons.iter().enumerate() {
                    if is_over(surface, text, *pos, *mouse) {
                        context.game().sounds.click.play();
                        return Some(idx);
                    }
                }
            }
            _ => {}
        }
        None
    }

    pub fn update(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        let surface = context.surface();
        for (pos, text, hovered) in &mut self.buttons {
            let color = if *hovered { HOVERED_TEXT_COLOR } else { TEXT_COLOR };
            surface.set_fill_style(&color.into());

            let px = pos.x as f64;
            let py = pos.y as f64;

            surface.set_font("2.5rem monospace");
            surface.set_text_align("center");
            surface.set_text_baseline("middle");
            surface.fill_text(text, px, py).unwrap();
        }
    }
}
