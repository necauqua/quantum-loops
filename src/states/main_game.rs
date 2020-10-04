use nalgebra::Point2;
use noise::{NoiseFn, Perlin};

use crate::{
    engine::{
        self,
        event::{Event, MouseButton},
        GameState,
        GameUpdate,
        StateTransition,
        util::SmoothChange,
    },
    level::GameLevel,
    QuantumLoops,
    states::main_menu::MainMenuState,
    states::pause::PauseState,
    TAU,
};
use crate::states::game_over::GameOverState;

pub const BG_COLOR: &str = "#ebf2f5";
pub const BG_LINE_COLOR: &str = "#d2e0fa";
pub const TEXT_COLOR: &str = "#119ad9";
pub const HOVERED_TEXT_COLOR: &str = "#0a5a80";
pub const ENERGY_BAR_COLOR: &str = "#93d6f5";

pub const JIGGLE_TIME: f32 = 0.25;

#[derive(Debug)]
pub struct Disruption {
    pub start: Point2<f32>,
    pub end: Point2<f32>,
    pub start_time: f64,
}

#[derive(Debug)]
pub struct MainGameState {
    game_level: GameLevel,
    current_ring: usize,
    particle_angle: f32,
    energy: SmoothChange,
    disruption: Option<Disruption>,
    noise: Perlin,
}

pub fn draw_background(context: &GameUpdate<QuantumLoops>, offset: Point2<f32>) {
    let surface = context.surface();
    let size = context.size();

    surface.set_fill_style(&BG_COLOR.into());
    surface.fill_rect(0.0, 0.0, size.x as f64, size.y as f64);

    surface.set_stroke_style(&BG_LINE_COLOR.into());
    surface.set_line_width(1.0);

    let mut i = offset.x % 100.0;
    while i < size.x {
        surface.begin_path();
        surface.move_to(i as f64, 0.0);
        surface.line_to(i as f64, size.y as f64);
        surface.stroke();
        i += 100.0;
    }
    i = offset.y % 100.0;
    while i < size.y {
        surface.begin_path();
        surface.move_to(0.0, i as f64);
        surface.line_to(size.x as f64, i as f64);
        surface.stroke();
        i += 100.0;
    }
}

impl MainGameState {
    pub fn new(game_level: GameLevel) -> Self {
        let energy = game_level.energy;
        Self {
            game_level,
            current_ring: 0,
            energy: SmoothChange::new(energy, 2.0),
            disruption: None,
            particle_angle: 0.0,
            noise: Perlin::new(),
        }
    }
}

const POWER_USED_PER_PIXEL_PER_SECOND: f32 = 1.0;

impl MainGameState {
    fn handle_disruption(&mut self, pos: Point2<f32>, context: &mut GameUpdate<QuantumLoops>) {
        if let Some(mut d) = self.disruption.take() {
            d.end = pos;

            let center = context.size() / 2.0;

            let dist = d.start.coords.metric_distance(&d.end.coords);
            let time = (engine::time() - d.start_time) as f32;

            let mut intersections = self.game_level.rings.iter_mut()
                .enumerate()
                .filter(|(_, r)|
                    r.disrupted_time <= 1.0 && r.intersects(center, &d))
                .collect::<Vec<_>>();

            let extras =
                if intersections.len() == 1 {
                    let (idx, ring) = &mut intersections[0];
                    ring.disrupted_time =
                        if *idx == self.current_ring {
                            ring.restore_time
                        } else {
                            JIGGLE_TIME
                        };
                    ring.base_energy
                } else {
                    intersections.iter_mut()
                        .map(|(_, r)| r.base_energy)
                        .sum()
                };
            self.energy.set(self.energy.get() - dist * time * POWER_USED_PER_PIXEL_PER_SECOND - extras);
        }
    }

    fn update_particle_level(&mut self, context: &mut GameUpdate<QuantumLoops>, play_sound: bool) -> bool {
        let jump_to =
            self.game_level.rings.iter()
                .enumerate()
                .filter(|(_, r)| r.disrupted_time <= 0.0)
                .max_by(|(_, r1), (_, r2)|
                    r1.base_energy.partial_cmp(&r2.base_energy).expect("NaN not allowed"))
                .map(|(idx, _)| idx);

        match jump_to {
            Some(idx) => {
                if self.current_ring != idx {
                    self.current_ring = idx;
                    if play_sound {
                        context.game().sounds.jump.play();
                    }
                }
                false
            }
            _ => true
        }
    }
}

impl GameState<QuantumLoops> for MainGameState {
    fn on_mounted(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        self.update_particle_level(context, false);
    }

    fn on_event(&mut self, event: Event, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match event {
            Event::KeyDown { code: 27, .. } => {
                return StateTransition::Push(Box::new(PauseState::new()));
            }
            Event::MouseDown { pos, button: MouseButton::Left } => {
                self.disruption = Some(Disruption {
                    start: pos,
                    end: pos,
                    start_time: engine::time(),
                })
            }
            Event::MouseMove { pos, buttons } => {
                if let Some(d) = self.disruption.as_mut() {
                    if buttons.contains(&MouseButton::Left) {
                        d.end = pos;
                    } else if engine::time() - d.start_time >= 0.01 {
                        self.handle_disruption(pos, context);
                    }
                }
            }
            Event::MouseUp { pos, button: MouseButton::Left } => {
                self.handle_disruption(pos, context);
            }
            _ => {}
        }
        StateTransition::None
    }


    fn update(&mut self, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.energy.update(context.delta_time());

        if self.energy.get() <= 0.0 {
            context.game().sounds.lose.play();
            let state = GameOverState::new("YOU LOST".into(), "red".into(), self.game_level.clone());
            return StateTransition::Set(Box::new(state));
        }

        self.particle_angle += (TAU * context.delta_time()) as f32;

        if self.update_particle_level(context, true) {
            context.game().sounds.win.play();
            let state = GameOverState::new("YOU WON".into(), TEXT_COLOR.into(), self.game_level.clone());
            return StateTransition::Set(Box::new(state));
        }

        // render:

        let size = context.size();
        let center = size / 2.0;

        draw_background(&context, center);

        let surface = context.surface();

        let w = size.x * self.energy.get_interp() / self.game_level.energy;
        surface.set_fill_style(&ENERGY_BAR_COLOR.into());
        surface.fill_rect(0.0, 0.0, w as f64, 15.0);

        // the dot idk
        surface.set_fill_style(&"black".into());
        surface.begin_path();
        surface.arc(center.x as f64, center.y as f64, 2.5, 0.0, TAU).unwrap();
        surface.fill();

        let min_dim = size.coords.min();

        let mut jiggling = 0.0;

        for (idx, ring) in self.game_level.rings.iter_mut().enumerate() {
            surface.set_stroke_style(&(&ring.color).into());

            let mut pos = center + ring.offset.coords;

            if ring.disrupted_time >= 0.0 && ring.disrupted_time <= JIGGLE_TIME {
                let offset = self.particle_angle as f64 * 5.0;
                pos.x += (self.noise.get([0.0, offset]) * 2.0 - 1.0) as f32 * 2.0;
                pos.y += (self.noise.get([offset, 0.0]) * 2.0 - 1.0) as f32 * 2.0;
                jiggling = ring.disrupted_time;
            }

            surface.set_global_alpha((1.0 - ring.disrupted_time / ring.restore_time) as f64);

            surface.set_line_width(ring.width as f64);
            surface.begin_path();
            surface.arc(pos.x as f64, pos.y as f64, (min_dim * ring.radius) as f64, 0.0, TAU).unwrap();
            surface.stroke();

            surface.set_global_alpha(1.0);

            if ring.disrupted_time >= 0.0 {
                ring.disrupted_time -= context.delta_time() as f32;
            }

            if idx == self.current_ring {
                let px = pos.x + min_dim * ring.radius * self.particle_angle.cos();
                let py = pos.y + min_dim * ring.radius * self.particle_angle.sin();

                surface.set_fill_style(&"blue".into());
                surface.begin_path();
                surface.arc(px as f64, py as f64, 7.0, 0.0, TAU).unwrap();
                surface.fill();
            }
        }

        let wrong_ring = &context.game().sounds.wrong_ring;
        if jiggling != 0.0 && !wrong_ring.playing() {
            wrong_ring.play();
        }

        if let Some(Disruption { start, end, .. }) = self.disruption.as_ref() {
            surface.set_stroke_style(&"red".into());
            surface.set_line_width(1.0);
            surface.begin_path();
            surface.move_to(start.x as f64, start.y as f64);
            surface.line_to(end.x as f64, end.y as f64);
            surface.stroke();
        }

        StateTransition::None
    }
}
