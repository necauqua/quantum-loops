use nalgebra::Point2;
use noise::{NoiseFn, Perlin};

use crate::{engine::{
    self,
    event::{Event, MouseButton},
    GameState,
    Context,
    StateTransition,
    util::SmoothChange,
}, level::GameLevel, QuantumLoops, TAU, sounds_enabled};
use crate::states::game_lost::GameLostState;
use crate::states::game_won::GameWonState;
use crate::engine::util::RemConversions;
use std::f32::consts::FRAC_PI_4;
use crate::states::pause::PauseState;

pub const BG_COLOR: &str = "#ebf2f5";
pub const BG_LINE_COLOR: &str = "#d2e0fa";
pub const TEXT_COLOR: &str = "#119ad9";
pub const DISABLED_TEXT_COLOR: &str = "#77868c";
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
enum GameStatus {
    Playing,
    Paused,
    Won { score: f32 },
    Lost,
}

#[derive(Debug)]
pub struct MainGameState {
    level: Option<GameLevel>,
    level_idx: usize,
    current_ring: usize,
    particle_angle: f32,
    game_status: GameStatus,
    energy: SmoothChange,
    disruption: Option<Disruption>,
    noise: Perlin,
}

pub fn draw_background(context: &Context<QuantumLoops>, offset: Point2<f32>) {
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

const POWER_USED_PER_PIXEL_PER_SECOND: f32 = 1.0;

impl MainGameState {
    pub fn new(level_idx: usize) -> Self {
        Self {
            level: None,
            level_idx,
            current_ring: 0,
            energy: SmoothChange::new(100.0, 50.0),
            disruption: None,
            particle_angle: 0.0,
            game_status: GameStatus::Playing,
            noise: Perlin::new(),
        }
    }

    pub fn resume(&mut self) {
        self.game_status = GameStatus::Playing;
    }

    pub fn level_idx(&self) -> usize {
        self.level_idx
    }

    fn check_level(&mut self, context: &mut Context<QuantumLoops>) -> Option<StateTransition<QuantumLoops>> {
        if self.level.is_some() {
            return None
        }
        let level = context.game_mut().get_level(self.level_idx);
        if level.is_none() {
            return Some(StateTransition::None);
        }
        self.energy.full_set(level.as_ref().unwrap().energy);
        self.level = level;
        None
    }

    fn handle_disruption(&mut self, pos: Point2<f32>, context: &mut Context<QuantumLoops>) {
        if let Some(mut d) = self.disruption.take() {
            d.end = pos;

            let center = context.size() / 2.0;

            let dist = d.start.coords.metric_distance(&d.end.coords);
            let time = (engine::time() - d.start_time) as f32;

            let level = self.level.as_mut().unwrap();
            let mut intersections = level.rings.iter_mut()
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

    fn update_particle_level(&mut self, context: &mut Context<QuantumLoops>, play_sound: bool) -> bool {
        let jump_to =
            self.level.as_mut().unwrap().rings.iter()
                .enumerate()
                .filter(|(_, r)| r.disrupted_time <= 0.0)
                .max_by(|(_, r1), (_, r2)|
                    r1.base_energy.partial_cmp(&r2.base_energy).expect("NaN not allowed"))
                .map(|(idx, _)| idx);

        match jump_to {
            Some(idx) => {
                if self.current_ring != idx {
                    self.current_ring = idx;
                    if play_sound && sounds_enabled() {
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
    fn on_pushed(&mut self, context: &mut Context<QuantumLoops>) {
        if self.check_level(context).is_some() {
            self.update_particle_level(context, false);
        }
    }

    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Some(transition) = self.check_level(context) {
            return transition;
        }
        match event {
            Event::KeyDown { code: 27, .. } => {
                self.game_status = GameStatus::Paused;
                return StateTransition::Pop;
            }
            Event::KeyDown { code: 82, .. } => {
                return StateTransition::Set(Box::new(MainGameState::new(self.level_idx)));
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

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Some(transition) = self.check_level(context) {
            return transition;
        }

        // render:

        let size = context.size();
        let center = size / 2.0;

        draw_background(&context, center);

        let surface = context.surface();

        let energy = self.energy.get_interp();
        let w = size.x * energy / self.level.as_mut().unwrap().energy;
        surface.set_fill_style(&ENERGY_BAR_COLOR.into());
        surface.fill_rect(0.0, 0.0, w as f64, 1.0.rem_to_pixels());

        surface.set_fill_style(&TEXT_COLOR.into());
        surface.set_font("0.9rem monospace");
        surface.fill_text(&format!("{:.2}", energy.max(0.0)),
                          1.6.rem_to_pixels(), 1.6.rem_to_pixels()).unwrap();

        let min_dim = size.coords.min();

        let mut jiggling = 0.0;

        let level = self.level.as_mut().unwrap();
        for (idx, ring) in level.rings.iter_mut().enumerate() {
            surface.set_stroke_style(&(&ring.color).into());

            let mut pos = center + ring.offset.coords * min_dim;

            if ring.disrupted_time >= 0.0 && ring.disrupted_time <= JIGGLE_TIME {
                let offset = self.particle_angle as f64 * 5.0;
                pos.x += (self.noise.get([0.0, offset]) * 2.0 - 1.0) as f32 * 2.0;
                pos.y += (self.noise.get([offset, 0.0]) * 2.0 - 1.0) as f32 * 2.0;
                jiggling = ring.disrupted_time;
            }

            let radius = min_dim * ring.radius;

            surface.set_global_alpha((1.0 - ring.disrupted_time / ring.restore_time) as f64);

            surface.set_line_width(ring.width as f64);
            surface.begin_path();
            surface.arc(pos.x as f64, pos.y as f64, (radius) as f64, 0.0, TAU).unwrap();
            surface.stroke();

            surface.set_global_alpha(1.0);

            let tpx = pos.x + (radius + 1.5.rem_to_pixels()) * FRAC_PI_4.cos();
            let tpy = pos.y + (radius + 1.5.rem_to_pixels()) * FRAC_PI_4.sin();

            surface.set_fill_style(&TEXT_COLOR.into());
            surface.set_font("0.9rem monospace");
            surface.fill_text(&format!("{:.2}", ring.base_energy), tpx as f64, tpy as f64).unwrap();

            if let GameStatus::Playing = self.game_status {
                if ring.disrupted_time >= 0.0 {
                    ring.disrupted_time -= context.delta_time() as f32;
                }
                if idx == self.current_ring {
                    let px = pos.x + radius * self.particle_angle.cos();
                    let py = pos.y + radius * self.particle_angle.sin();

                    surface.set_fill_style(&"blue".into());
                    surface.begin_path();
                    surface.arc(px as f64, py as f64, 7.0, 0.0, TAU).unwrap();
                    surface.fill();
                }
            }
        }

        if let GameStatus::Playing = self.game_status {
            let wrong_ring = &context.game().sounds.wrong_ring;
            if jiggling != 0.0 && !wrong_ring.playing() && sounds_enabled() {
                wrong_ring.play();
            }
        }

        if let Some(Disruption { start, end, .. }) = self.disruption.as_ref() {
            surface.set_stroke_style(&"red".into());
            surface.set_line_width(1.0);
            surface.begin_path();
            surface.move_to(start.x as f64, start.y as f64);
            surface.line_to(end.x as f64, end.y as f64);
            surface.stroke();
        }

        self.energy.update(context.delta_time());

        if self.energy.get() <= 0.0 {
            self.game_status = GameStatus::Lost;
            return StateTransition::Pop;
        }

        if self.update_particle_level(context, true) {
            let level = self.level.as_ref().unwrap();

            let free = level.energy - level.rings
                .iter()
                .map(|r| r.base_energy)
                .sum::<f32>();

            self.game_status = GameStatus::Won {
                score: (1.0 - (free - self.energy.get()) / free) * 100.0,
            };
            return StateTransition::Pop;
        }

        if let GameStatus::Playing = self.game_status {
            self.particle_angle += (TAU * context.delta_time()) as f32;
        }

        StateTransition::None
    }

    fn on_popped(self: Box<Self>, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match self.game_status {
            GameStatus::Won { score } => {
                if sounds_enabled() {
                    context.game().sounds.win.play();
                }
                StateTransition::Push(Box::new(GameWonState::new(*self, score)))
            },
            GameStatus::Lost => {
                if sounds_enabled() {
                    context.game().sounds.lose.play();
                }
                StateTransition::Push(Box::new(GameLostState::new(*self)))
            },
            GameStatus::Paused => {
                StateTransition::Push(Box::new(PauseState::new(*self)))
            }
            _ => StateTransition::None,
        }
    }
}
