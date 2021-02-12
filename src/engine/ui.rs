use std::{
    borrow::Cow,
    fmt::{Debug, Formatter},
};

use nalgebra::Vector2;

use crate::{
    engine::{
        event::{Event, MouseButton},
        Context,
    },
    states::main_game::{DISABLED_TEXT_COLOR, HOVERED_TEXT_COLOR, TEXT_COLOR},
    QuantumLoops,
};

pub struct Text {
    pub pos: Vector2<f64>,
    pub text: Cow<'static, str>,
    size: f64,
    font: String,
}

impl Debug for Text {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Text")
            .field("pos", &self.pos)
            .field("text", &self.text)
            .field("size", &self.size)
            .finish()
    }
}

impl Text {
    pub fn empty() -> Text {
        Self::new("".into())
    }

    pub fn new(text: Cow<'static, str>) -> Text {
        Self {
            pos: [0.0, 0.0].into(),
            text,
            size: 2.5,
            font: "2.5rem monospace".into(),
        }
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.set_size(size);
        self
    }

    pub fn set_size(&mut self, size: f64) {
        self.size = size;
        self.font = format!("{}rem monospace", size);
    }

    pub fn compute_size(&self, context: &mut Context<QuantumLoops>) -> (f64, f64) {
        let surface = context.surface().context();
        surface.set_font(&self.font);
        let dim = surface.measure_text(&self.text).unwrap();
        (dim.width(), context.rem_to_px(self.size))
    }

    pub fn is_over(&self, pos: Vector2<f64>, context: &mut Context<QuantumLoops>) -> bool {
        let (w, h) = self.compute_size(context);
        pos.x >= self.pos.x - w / 2.0
            && pos.x <= self.pos.x + w / 2.0
            && pos.y >= self.pos.y - h / 2.0
            && pos.y <= self.pos.y + h / 2.0
    }

    pub fn on_update(
        &mut self,
        context: &mut Context<QuantumLoops>,
        pos: Vector2<f64>,
        color: &str,
    ) {
        let surface = context.surface().context();

        self.pos = pos;

        surface.set_fill_style(&color.into());
        surface.set_font(&self.font);
        surface.fill_text(&self.text, pos.x, pos.y).unwrap();
    }
}

#[derive(Debug)]
pub struct Button {
    pub text: Text,
    pub enabled: bool,
    hovered: bool,
    last_touch: Option<Vector2<f64>>,
}

impl Button {
    pub fn empty() -> Self {
        Self::new("".into())
    }

    pub fn new(text: Cow<'static, str>) -> Self {
        Self {
            text: Text::new(text),
            hovered: false,
            enabled: true,
            last_touch: None,
        }
    }

    pub fn with_size(mut self, size: f64) -> Self {
        self.text.set_size(size);
        self
    }

    pub fn set_text(&mut self, text: Cow<'static, str>) {
        self.text.text = text;
    }

    fn handle_press(&mut self, pos: Vector2<f64>, context: &mut Context<QuantumLoops>) -> bool {
        if self.text.is_over(pos, context) {
            context.game.sounds.click.play();
            true
        } else {
            false
        }
    }

    pub fn on_event(&mut self, event: &Event, context: &mut Context<QuantumLoops>) -> bool {
        if !self.enabled {
            return false;
        }
        match event {
            Event::MouseMove { pos, .. } => {
                let over = self.text.is_over(*pos, context);
                if !self.hovered && over {
                    context.game.sounds.hover.play();
                }
                self.hovered = over;
                false
            }
            Event::MouseUp {
                pos,
                button: MouseButton::Left,
            } => self.handle_press(*pos, context),
            Event::TouchStart { touches } => {
                self.last_touch = touches.get(0).copied();
                false
            }
            Event::TouchMove { touches } => {
                self.last_touch = touches.get(0).copied();
                false
            }
            Event::TouchEnd { touches } if touches.len() <= 1 => {
                self.hovered = false;
                if let Some(pos) = touches.get(0).copied().or(self.last_touch) {
                    self.handle_press(pos, context)
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    pub fn on_update(&mut self, context: &mut Context<QuantumLoops>, pos: Vector2<f64>) {
        self.text.on_update(
            context,
            pos,
            if !self.enabled {
                DISABLED_TEXT_COLOR
            } else if self.hovered {
                HOVERED_TEXT_COLOR
            } else {
                TEXT_COLOR
            },
        );
    }
}
