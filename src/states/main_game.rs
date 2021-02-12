use nalgebra::Vector2;
use noise::{NoiseFn, Perlin};

use crate::{
    engine::{
        self,
        event::{Event, MouseButton},
        util::SmoothChange,
        Context, GameState, StateTransition,
    },
    level::GameLevel,
    level::StoredData,
    states::game_lost::GameLostState,
    states::game_won::GameWonState,
    states::pause::PauseState,
    QuantumLoops,
};
use std::f64::consts::{FRAC_PI_4, TAU};

pub const BG_COLOR: &str = "#ebf2f5";
pub const BG_LINE_COLOR: &str = "#d2e0fa";
pub const TEXT_COLOR: &str = "#119ad9";
pub const DISABLED_TEXT_COLOR: &str = "#77868c";
pub const HOVERED_TEXT_COLOR: &str = "#0a5a80";
pub const ENERGY_BAR_COLOR: &str = "#93d6f5";

pub const JIGGLE_TIME: f64 = 0.25;

#[derive(Debug)]
enum DisruptionCause {
    Mouse,
    Touch,
}

#[derive(Debug)]
pub struct Disruption {
    pub start: Vector2<f64>,
    pub end: Vector2<f64>,
    pub start_time: f64,
    cause: DisruptionCause,
}

#[derive(Debug)]
enum GameStatus {
    Playing,
    Paused,
    Won { score: f64 },
    Lost,
}

#[derive(Debug)]
pub struct MainGameState {
    level: Option<GameLevel>,
    level_idx: usize,
    current_ring: usize,
    particle_angle: f64,
    game_status: GameStatus,
    energy: SmoothChange,
    disruption: Option<Disruption>,
    noise: Perlin,
}

pub fn draw_background(context: &Context<QuantumLoops>, offset: Vector2<f64>) {
    let size = context.surface().size();
    let surface = context.surface().context();

    surface.set_fill_style(&BG_COLOR.into());
    surface.fill_rect(0.0, 0.0, size.x, size.y);

    surface.set_stroke_style(&BG_LINE_COLOR.into());
    surface.set_line_width(1.0);

    let mut i = offset.x % 100.0;
    while i < size.x {
        surface.begin_path();
        surface.move_to(i, 0.0);
        surface.line_to(i, size.y);
        surface.stroke();
        i += 100.0;
    }
    i = offset.y % 100.0;
    while i < size.y {
        surface.begin_path();
        surface.move_to(0.0, i);
        surface.line_to(size.x, i);
        surface.stroke();
        i += 100.0;
    }
}

const POWER_USED_PER_PIXEL_PER_SECOND: f64 = 1.0;

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

    fn check_level(
        &mut self,
        context: &mut Context<QuantumLoops>,
    ) -> Option<StateTransition<QuantumLoops>> {
        if self.level.is_some() {
            return None;
        }
        let level = context.game.get_level(self.level_idx);
        if level.is_none() {
            return Some(StateTransition::None);
        }
        self.energy.set_raw(level.as_ref().unwrap().energy);
        self.level = level;
        None
    }

    fn start_disruption(&mut self, pos: Vector2<f64>, cause: DisruptionCause) {
        self.disruption = Some(Disruption {
            start: pos,
            end: pos,
            start_time: engine::time(),
            cause,
        })
    }

    fn update_disruption(
        &mut self,
        pos: Vector2<f64>,
        dragging: bool,
        context: &mut Context<QuantumLoops>,
    ) {
        if let Some(d) = self.disruption.as_mut() {
            if dragging {
                d.end = pos;
            } else if engine::time() - d.start_time >= 0.01 {
                self.finish_disruption(Some(pos), context);
            }
        }
    }

    fn finish_disruption(
        &mut self,
        pos: Option<Vector2<f64>>,
        context: &mut Context<QuantumLoops>,
    ) {
        if let Some(mut d) = self.disruption.take() {
            if let Some(pos) = pos {
                d.end = pos;
            }

            let center = context.surface().size() / 2.0;

            let dist = d.start.metric_distance(&d.end);
            let time = engine::time() - d.start_time;

            let level = self.level.as_mut().unwrap();
            let mut intersections = level
                .rings
                .iter_mut()
                .enumerate()
                .filter(|(_, r)| r.disrupted_time <= 1.0 && r.intersects(center, &d))
                .collect::<Vec<_>>();

            let extras = if intersections.len() == 1 {
                let (idx, ring) = &mut intersections[0];
                ring.disrupted_time = if *idx == self.current_ring {
                    ring.restore_time
                } else {
                    JIGGLE_TIME
                };
                ring.base_energy
            } else {
                intersections.iter_mut().map(|(_, r)| r.base_energy).sum()
            };

            self.energy
                .set(self.energy.get() - dist * time * POWER_USED_PER_PIXEL_PER_SECOND - extras);
        }
    }

    fn update_particle_level(
        &mut self,
        context: &mut Context<QuantumLoops>,
        play_sound: bool,
    ) -> bool {
        let jump_to = self
            .level
            .as_mut()
            .unwrap()
            .rings
            .iter()
            .enumerate()
            .filter(|(_, r)| r.disrupted_time <= 0.0)
            .max_by(|(_, r1), (_, r2)| {
                r1.base_energy
                    .partial_cmp(&r2.base_energy)
                    .expect("NaN not allowed")
            })
            .map(|(idx, _)| idx);

        if let Some(idx) = jump_to {
            if self.current_ring != idx {
                self.current_ring = idx;
                if play_sound {
                    context.game.sounds.jump.play();
                }
            }
            false
        } else {
            true
        }
    }
}

impl GameState<QuantumLoops> for MainGameState {
    fn on_pushed(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if self.check_level(context).is_some() {
            self.update_particle_level(context, false);
        }
        StateTransition::None
    }

    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        if let Some(transition) = self.check_level(context) {
            return transition;
        }
        log::debug!("event {:?}", event);
        match &event {
            Event::KeyDown { code: 27, .. } => {
                self.game_status = GameStatus::Paused;
                return StateTransition::Pop;
            }
            Event::KeyDown { code: 82, .. } => {
                return StateTransition::set(MainGameState::new(self.level_idx));
            }

            Event::MouseDown {
                pos,
                button: MouseButton::Left,
            } => self.start_disruption(*pos, DisruptionCause::Mouse),

            Event::MouseMove { pos, buttons } => {
                if let Some(DisruptionCause::Mouse) = self.disruption.as_ref().map(|d| &d.cause) {
                    self.update_disruption(
                        *pos,
                        buttons.contains(&MouseButton::Left),
                        context,
                    )
                }
            }

            Event::MouseUp {
                pos,
                button: MouseButton::Left,
            } => {
                if let Some(DisruptionCause::Mouse) = self.disruption.as_ref().map(|d| &d.cause) {
                    self.finish_disruption(Some(*pos), context)
                }
            }

            Event::TouchStart { touches } if touches.len() == 1 => {
                self.start_disruption(touches[0], DisruptionCause::Touch)
            }

            Event::TouchMove { touches } if touches.len() == 1 => {
                if let Some(DisruptionCause::Touch) = self.disruption.as_ref().map(|d| &d.cause) {
                    self.update_disruption(touches[0], true, context)
                }
            }

            Event::TouchEnd { touches } if touches.len() <= 1 => {
                if let Some(DisruptionCause::Touch) = self.disruption.as_ref().map(|d| &d.cause) {
                    self.finish_disruption(touches.get(0).copied(), context)
                }
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

        let size = context.surface().size();
        let center = size / 2.0;

        draw_background(&context, center);

        let surface = context.surface().context();

        let energy = self.energy.get_interp();
        let w = size.x * energy / self.level.as_mut().unwrap().energy;
        surface.set_fill_style(&ENERGY_BAR_COLOR.into());
        surface.fill_rect(0.0, 0.0, w, context.rem_to_px(1.0));

        surface.set_fill_style(&TEXT_COLOR.into());
        surface.set_font("0.9rem monospace");
        surface
            .fill_text(
                &format!("{:.2}", energy.max(0.0)),
                context.rem_to_px(1.6),
                context.rem_to_px(1.6),
            )
            .unwrap();

        let min_dim = size.min();

        let mut jiggling = 0.0;

        let level = self.level.as_mut().unwrap();
        for (idx, ring) in level.rings.iter_mut().enumerate() {
            surface.set_stroke_style(&(&ring.color).into());

            let mut pos = center + ring.offset * min_dim;

            if ring.disrupted_time > 0.0 && ring.disrupted_time <= JIGGLE_TIME {
                let offset = self.particle_angle * 5.0;
                pos.x += (self.noise.get([0.0, offset]) * 2.0 - 1.0) * 2.0;
                pos.y += (self.noise.get([offset, 0.0]) * 2.0 - 1.0) * 2.0;
                jiggling = ring.disrupted_time;
            }

            let radius = min_dim * ring.radius;

            surface.set_global_alpha(1.0 - ring.disrupted_time / ring.restore_time);

            surface.set_line_width(ring.width);
            surface.begin_path();
            surface.arc(pos.x, pos.y, radius, 0.0, TAU).unwrap();
            surface.stroke();

            surface.set_global_alpha(1.0);

            let tpx = pos.x + (radius + context.rem_to_px(1.5)) * FRAC_PI_4.cos();
            let tpy = pos.y + (radius + context.rem_to_px(1.5)) * FRAC_PI_4.sin();

            surface.set_fill_style(&TEXT_COLOR.into());
            surface.set_font("0.9rem monospace");
            surface
                .fill_text(&format!("{:.2}", ring.base_energy), tpx, tpy)
                .unwrap();

            if let GameStatus::Playing = self.game_status {
                if ring.disrupted_time > 0.0 {
                    ring.disrupted_time -= context.delta_time();
                }
                if idx == self.current_ring {
                    let px = pos.x + radius * self.particle_angle.cos();
                    let py = pos.y + radius * self.particle_angle.sin();

                    surface.set_fill_style(&"blue".into());
                    surface.begin_path();
                    surface.arc(px, py, 7.0, 0.0, TAU).unwrap();
                    surface.fill();
                }
            }
        }

        if let GameStatus::Playing = self.game_status {
            if jiggling != 0.0 {
                context.game.sounds.wrong_ring.play_unique();
            }
        }

        if let Some(Disruption { start, end, .. }) = self.disruption.as_ref() {
            surface.set_stroke_style(&"red".into());
            surface.set_line_width(1.0);
            surface.begin_path();
            surface.move_to(start.x, start.y);
            surface.line_to(end.x, end.y);
            surface.stroke();
        }

        self.energy.update(context.delta_time());

        if self.energy.get() <= 0.0 {
            self.game_status = GameStatus::Lost;
            return StateTransition::Pop;
        }

        if let GameStatus::Playing = self.game_status {
            if self.update_particle_level(context, true) {
                let level = self.level.as_ref().unwrap();

                let storage = context.storage();
                if storage.unlocked_level < self.level_idx + 1 {
                    let new_storage = StoredData {
                        unlocked_level: self.level_idx + 1,
                        ..storage.clone()
                    };
                    context.set_storage(new_storage);
                }

                let free = level.energy - level.rings.iter().map(|r| r.base_energy).sum::<f64>();

                self.game_status = GameStatus::Won {
                    score: (1.0 - (free - self.energy.get()) / free) * 100.0,
                };
                return StateTransition::Pop;
            }
            self.particle_angle += TAU * context.delta_time();
        }

        StateTransition::None
    }

    fn on_popped(
        self: Box<Self>,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        match self.game_status {
            GameStatus::Won { score } => {
                context.game.sounds.win.play();
                StateTransition::push(GameWonState::new(*self, score))
            }
            GameStatus::Lost => {
                context.game.sounds.lose.play();
                StateTransition::push(GameLostState::new(*self))
            }
            GameStatus::Paused => StateTransition::push(PauseState::new(*self)),
            _ => StateTransition::None,
        }
    }
}
