use noise::{NoiseFn, Perlin};
use wasm_bindgen::prelude::*;

use engine::{
    *,
    event::{Event, MouseButton},
    event::Event::MouseDown,
    sound::Music,
    util::setup_panic_hook,
};

mod engine;

const TAU: f64 = 2.0 * std::f64::consts::PI;

struct QuantumLoops {
    background: Music,
}

#[derive(Debug)]
struct MainMenuState {
    noise: Perlin,
    offset: f64,
}

#[derive(Debug)]
struct PauseState;

#[derive(Debug)]
struct Particle {
    angle: f64,
    level: u64,
}

#[derive(Debug)]
struct Disruption {
    start: (f64, f64),
    end: (f64, f64),
}

#[derive(Debug)]
struct PlayingState {
    disruption: Option<Disruption>,
    particle: Particle,
}

#[derive(Debug)]
struct FinishedState;

const BG_COLOR: &str = "#ebf2f5";

fn draw_background(context: &GameUpdate<QuantumLoops>, (x, y): (f64, f64)) {
    let surface = context.surface();
    let (width, height) = context.size();

    surface.set_fill_style(&BG_COLOR.into());
    surface.fill_rect(0.0, 0.0, width, height);

    surface.set_stroke_style(&"#d2e0fa".into());
    surface.set_line_width(1.0);

    let mut i = x % 100.0;
    while i < width {
        surface.begin_path();
        surface.move_to(i, 0.0);
        surface.line_to(i, height);
        surface.stroke();
        i += 100.0;
    }
    i = y % 100.0;
    while i < height {
        surface.begin_path();
        surface.move_to(0.0, i);
        surface.line_to(width, i);
        surface.stroke();
        i += 100.0;
    }
}

impl MainMenuState {
    fn new() -> Self {
        Self {
            noise: Perlin::new(),
            offset: 0.0,
        }
    }
}

impl GameState<QuantumLoops> for MainMenuState {
    fn on_mounted(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        let bg = &context.game().background;
        bg.set_volume(0.01);
        bg.play();
    }

    fn update(&mut self, mut context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        for event in context.events() {
            if let MouseDown { button: MouseButton::Left, .. } = event {
                context.game().background.play();
                return StateTransition::Set(Box::new(PlayingState {
                    particle: Particle { angle: 0.0, level: 0 },
                    disruption: None,
                }));
            }
        }
        let nx = (self.noise.get([0.0, self.offset]) * 2.0 - 1.0) * 50.0;
        let ny = (self.noise.get([self.offset, 0.0]) * 2.0 - 1.0) * 50.0;
        draw_background(&context, (nx, ny));
        self.offset += context.delta_time() / 5.0;

        let surface = context.surface();
        let (width, height) = context.size();

        surface.set_text_align("center");
        surface.set_text_baseline("middle");
        surface.set_fill_style(&"black".into());
        surface.set_font("5rem monospace");

        surface.fill_text("CLICK ANYWHERE TO PLAY", width / 2.0, height / 2.0).unwrap();

        StateTransition::None
    }
}

impl GameState<QuantumLoops> for PauseState {
    fn on_mounted(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        let surface = context.surface();
        surface.set_text_align("center");
        surface.set_text_baseline("middle");
        surface.set_fill_style(&"black".into());
        surface.set_font("5rem monospace");
        let (width, height) = context.size();
        surface.fill_text("PAUSED", width / 2.0, height / 2.0).unwrap();
    }

    fn update(&mut self, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        for event in context.events() {
            if let MouseDown { button: MouseButton::Left, .. } = event {
                return StateTransition::Pop;
            }
        }
        StateTransition::None
    }
}

impl GameState<QuantumLoops> for FinishedState {}

impl GameState<QuantumLoops> for PlayingState {
    fn update(&mut self, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        for event in context.events() {
            match event {
                Event::KeyDown { code: 27, .. } => {
                    return StateTransition::Push(Box::new(PauseState));
                }
                Event::MouseDown { button: MouseButton::Left, x, y } => {
                    self.disruption = Some(Disruption {
                        start: (x as f64, y as f64),
                        end: (x as f64, y as f64),
                    })
                }
                Event::MouseMove { x, y, buttons } => {
                    if buttons.contains(&MouseButton::Left) {
                        if let Some(disruption) = self.disruption.as_mut() {
                            disruption.end = (x as f64, y as f64);
                        }
                    }
                }
                _ => {}
            }
        }

        let (width, height) = context.size();

        let cx = width / 2.0;
        let cy = height / 2.0;

        draw_background(&context, (cx, cy));

        let surface = context.surface();

        let min_dim = width.min(height);

        let radius1 = 0.25 * min_dim;
        let radius2 = 0.15 * min_dim;

        surface.set_fill_style(&"black".into());
        surface.begin_path();
        surface.arc(cx, cy, 2.5, 0.0, TAU).unwrap();
        surface.fill();

        surface.set_stroke_style(&"black".into());
        surface.set_line_width(1.0);
        surface.begin_path();
        surface.arc(cx, cy, radius1, 0.0, TAU).unwrap();
        surface.stroke();
        surface.begin_path();
        surface.arc(cx, cy, radius2, 0.0, TAU).unwrap();
        surface.stroke();

        if let Some(Disruption { start, end }) = self.disruption.as_ref() {
            surface.set_stroke_style(&"red".into());
            surface.set_line_width(1.0);
            surface.begin_path();
            surface.move_to(start.0, start.1);
            surface.line_to(end.0, end.1);
            surface.stroke();
        }

        let r = match self.particle.level {
            1 => radius1,
            _ => radius2,
        };

        let px = cx + r * self.particle.angle.cos();
        let py = cy + r * self.particle.angle.sin();

        surface.set_fill_style(&"blue".into());
        surface.begin_path();
        surface.arc(px, py, 10.0, 0.0, TAU).unwrap();
        surface.fill();

        self.particle.angle += TAU * context.delta_time();

        StateTransition::None
    }
}

impl Game for QuantumLoops {
    fn load(resources: Resources) -> (Self, Box<dyn GameState<QuantumLoops>>) {
        let global = QuantumLoops {
            background: resources.load_music("assets/background.mp3").looped()
        };
        (global, Box::new(MainMenuState::new()))
    }
}

#[wasm_bindgen]
pub fn main() {
    wasm_logger::init(Default::default());
    setup_panic_hook();

    QuantumLoops::run();
}
