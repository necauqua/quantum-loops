use nalgebra::Vector2;
use noise::{NoiseFn, Perlin};

use crate::{
    engine::{self, *},
    QuantumLoops,
    states::main_game::{draw_background, MainGameState},
};
use crate::engine::event::Event;
use crate::states::level_select::LevelSelectState;
use crate::ui::Buttons;

#[derive(Debug)]
pub struct Background {
    noise: Perlin,
    offset: f64,
}

impl Background {
    pub fn new() -> Self {
        Self {
            noise: Perlin::new(),
            offset: 0.0,
        }
    }

    pub fn update(&mut self, context: &GameUpdate<QuantumLoops>) {
        let nx = (self.noise.get([0.0, self.offset]) * 2.0 - 1.0) * 50.0;
        let ny = (self.noise.get([self.offset, 0.0]) * 2.0 - 1.0) * 50.0;

        draw_background(&context, [nx as f32, ny as f32].into());

        self.offset += context.delta_time() / 5.0;
    }
}

#[derive(Debug)]
pub struct MainMenuState {
    background: Background,
    buttons: Buttons,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            background: Background::new(),
            buttons: Buttons::new(),
        }
    }
}

impl GameState<QuantumLoops> for MainMenuState {
    fn on_mounted(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        let bg = &context.game().sounds.background;
        bg.set_volume(0.01);
        bg.play();

        let size = context.size();
        let center = size / 2.0;
        let offset: Vector2<f32> = [0.0, 100.0].into();

        self.buttons.add_button(center - offset, "Play".into());
        self.buttons.add_button(center, "Options".into());
        self.buttons.add_button(center + offset, "Exit".into());
    }

    fn on_event(&mut self, event: Event, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match self.buttons.on_event(&event, context) {
            Some(0) => StateTransition::Set(Box::new(LevelSelectState::new())),
            Some(1) => {
                // TODO options
                // return StateTransition::Set(Box::new(OptionsState::new()));
                StateTransition::None
            }
            Some(2) => {
                engine::window().close().unwrap();
                StateTransition::None
            }
            _ => StateTransition::None
        }
    }

    fn update(&mut self, context: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.background.update(context);
        self.buttons.update(context);
        StateTransition::None
    }
}
