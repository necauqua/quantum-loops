use nalgebra::Vector2;
use noise::{NoiseFn, Perlin};

use crate::{engine::{
    self,
    *,
    event::Event
}, level::StoredData, QuantumLoops, states::level_select::LevelMenuState, states::main_game::draw_background, states::tutorial::TutorialState, ui::Button, music_enabled};
use crate::engine::util::RemConversions;
use crate::states::options::OptionsState;
use crate::states::scores::ScoresState;

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

    pub fn render(&mut self, context: &Context<QuantumLoops>) {
        let nx = (self.noise.get([0.0, self.offset]) * 2.0 - 1.0) * 50.0;
        let ny = (self.noise.get([self.offset, 0.0]) * 2.0 - 1.0) * 50.0;

        draw_background(&context, [nx as f32, ny as f32].into());

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
    fn on_pushed(&mut self, context: &mut Context<QuantumLoops>) {
        if music_enabled() {
            context.game().sounds.background.play();
        }
    }

    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Event::MouseDown {..} = event {
            if music_enabled() {
                context.game().sounds.background.play();
            }
        }
        if self.play.on_event(&event, context) {
            return StateTransition::Set(
                if engine::get_data::<StoredData>().passed_tutorial {
                    Box::new(LevelMenuState::new())
                } else {
                    Box::new(TutorialState::new())
                });
        } else if self.scores.on_event(&event, context) {
            return StateTransition::Push(Box::new(ScoresState::new()));
        } else if self.options.on_event(&event, context) {
            return StateTransition::Set(Box::new(OptionsState::new()));
        } else if self.exit.on_event(&event, context) {
            engine::window().close().unwrap();
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let center = context.size() / 2.0;
        let offset: Vector2<f32> = [0.0, 2.5.rem_to_pixels()].into();

        self.play.pos = center - offset * 2.0;
        self.scores.pos = center - offset;
        self.options.pos = center;
        self.exit.pos = center + offset;

        self.background.render(context);
        self.play.render(context);
        self.scores.render(context);
        self.options.render(context);
        self.exit.render(context);

        StateTransition::None
    }
}
