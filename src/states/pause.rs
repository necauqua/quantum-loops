use crate::{
    engine::{event::Event, ui::Button, Context, GameState, StateTransition},
    states::{
        level_select::LevelMenuState,
        main_game::{MainGameState, TEXT_COLOR},
    },
    QuantumLoops,
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
            retry: Button::new("Retry".into()).with_size(1.5),
            level_menu: Button::new("Level Menu".into()).with_size(1.5),
            resume: Button::new("Resume".into()),
        }
    }
}

impl GameState<QuantumLoops> for PauseState {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 27, .. } = event {
            StateTransition::Pop
        } else if self.retry.on_event(&event, context) {
            StateTransition::set(MainGameState::new(self.game_state.level_idx()))
        } else if self.resume.on_event(&event, context) {
            StateTransition::Pop
        } else if self.level_menu.on_event(&event, context) {
            StateTransition::set(LevelMenuState::new())
        } else {
            StateTransition::None
        }
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.game_state.on_update(context);

        let center = context.surface().size() / 2.0;
        let surface = context.surface().context();
        surface.set_fill_style(&TEXT_COLOR.into());
        surface.set_font("5rem monospace");
        surface
            .fill_text("PAUSED", center.x, center.y - context.rem_to_px(2.5))
            .unwrap();

        self.retry.on_update(
            context,
            [center.x, center.y + context.rem_to_px(1.5)].into(),
        );
        self.level_menu.on_update(
            context,
            [center.x, center.y + context.rem_to_px(3.0)].into(),
        );
        self.resume.on_update(
            context,
            [center.x, center.y + context.rem_to_px(6.0)].into(),
        );

        StateTransition::None
    }

    fn on_popped(self: Box<Self>, _: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let mut game_state = self.game_state;
        game_state.resume();
        StateTransition::push(game_state)
    }
}
