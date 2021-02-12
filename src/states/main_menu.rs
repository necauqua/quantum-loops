use noise::{NoiseFn, Perlin};

use crate::{
    engine::{self, event::Event, ui::Button, *},
    states::{
        level_select::LevelMenuState, main_game::draw_background, options::OptionsState,
        scores::ScoresState, tutorial::TutorialState,
    },
    QuantumLoops,
};
use nalgebra::Vector2;

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

    pub fn on_update(&mut self, context: &Context<QuantumLoops>) {
        let nx = (self.noise.get([0.0, self.offset]) * 2.0 - 1.0) * 50.0;
        let ny = (self.noise.get([self.offset, 0.0]) * 2.0 - 1.0) * 50.0;

        draw_background(&context, [nx, ny].into());

        self.offset += context.delta_time() / 5.0;
    }
}

#[derive(Debug)]
pub struct MainMenuState {
    background: Background,
    play: Button,
    scores: Button,
    options: Button,
    exit: Button,
}

impl MainMenuState {
    pub fn new() -> Self {
        Self {
            background: Background::new(),
            play: Button::new("Play".into()),
            scores: Button::new("Scores".into()),
            options: Button::new("Options".into()),
            exit: Button::new("Exit".into()),
        }
    }
}

impl GameState<QuantumLoops> for MainMenuState {
    fn on_pushed(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        context.sound_context_mut().sound_mask = context.storage().get_enabled_sounds();
        StateTransition::None
    }

    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        if self.play.on_event(&event, context) {
            return StateTransition::Set(if context.storage().passed_tutorial {
                Box::new(LevelMenuState::new())
            } else {
                Box::new(TutorialState::new())
            });
        } else if self.scores.on_event(&event, context) {
            return StateTransition::push(ScoresState::new());
        } else if self.options.on_event(&event, context) {
            return StateTransition::set(OptionsState::new());
        } else if self.exit.on_event(&event, context) {
            engine::window().history().unwrap().back().unwrap();
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let center = context.surface().size() / 2.0;
        let offset: Vector2<f64> = [0.0, context.rem_to_px(2.5)].into();

        context.game.sounds.background.play_unique();

        self.background.on_update(context);
        self.play.on_update(context, center - offset * 2.0);
        self.scores.on_update(context, center - offset);
        self.options.on_update(context, center);
        self.exit.on_update(context, center + offset);

        StateTransition::None
    }
}
