use crate::{
    engine::{
        Context,
        event::Event,
        GameState,
        StateTransition,
    },
    engine::util::RemConversions,
    QuantumLoops,
    states::{
        level_select::LevelMenuState,
        main_game::{MainGameState, TEXT_COLOR}
    },
    ui::Button
};

#[derive(Debug)]
pub struct PauseState {
    game_state: MainGameState,
    retry: Button,
    level_menu: Button,
    resume: Button,
}

impl PauseState {
    pub fn new(game_state: MainGameState) -> Self {
        Self {
            game_state,
            retry: Button::new("Retry".into())
                .with_size(1.5),
            level_menu: Button::new("Level Menu".into())
                .with_size(1.5),
            resume: Button::new("Resume".into()),
        }
    }
}

impl GameState<QuantumLoops> for PauseState {

    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 27, .. } = event {
            StateTransition::Pop
        } else if self.retry.on_event(&event, context) {
            StateTransition::Set(Box::new(MainGameState::new(self.game_state.level_idx())))
        } else if self.resume.on_event(&event, context) {
            StateTransition::Pop
        } else if self.level_menu.on_event(&event, context) {
            StateTransition::Set(Box::new(LevelMenuState::new()))
        } else {
            StateTransition::None
        }
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.game_state.on_update(context);

        let surface = context.surface();
        surface.set_fill_style(&TEXT_COLOR.into());
        surface.set_font("5rem monospace");
        let center = context.size() / 2.0;
        surface.fill_text("PAUSED", center.x as f64, center.y as f64 - 2.5.rem_to_pixels()).unwrap();

        self.retry.pos = [center.x, center.y + 1.5.rem_to_pixels()].into();
        self.level_menu.pos = [center.x, center.y + 3.0.rem_to_pixels()].into();
        self.resume.pos = [center.x, center.y + 6.0.rem_to_pixels()].into();

        self.retry.render(context);
        self.resume.render(context);
        self.level_menu.render(context);

        StateTransition::None
    }

    fn on_popped(self: Box<Self>, _: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let mut game_state = self.game_state;
        game_state.resume();
        StateTransition::Push(Box::new(game_state))
    }
}
