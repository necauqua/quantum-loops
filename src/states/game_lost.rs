use crate::engine::ui::Button;
use crate::states::level_select::LevelMenuState;
use crate::{
    engine::{event::Event, Context, GameState, StateTransition},
    states::main_game::MainGameState,
    QuantumLoops,
};

#[derive(Debug)]
pub struct GameLostState {
    game_state: MainGameState,
    level_menu: Button,
    retry: Button,
}

impl GameLostState {
    pub fn new(game_state: MainGameState) -> Self {
        Self {
            game_state,
            level_menu: Button::new("Level Menu".into()).with_size(1.5),
            retry: Button::new("Retry".into()),
        }
    }
}

impl GameState<QuantumLoops> for GameLostState {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 82, .. } = event {
            return StateTransition::set(MainGameState::new(self.game_state.level_idx()));
        }
        if self.level_menu.on_event(&event, context) {
            StateTransition::set(LevelMenuState::new())
        } else if self.retry.on_event(&event, context) {
            StateTransition::set(MainGameState::new(self.game_state.level_idx()))
        } else {
            StateTransition::None
        }
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.game_state.on_update(context);

        let surface = context.surface().context();
        surface.set_fill_style(&"red".into());
        surface.set_font("5rem monospace");
        let center = context.surface().size() / 2.0;

        surface
            .fill_text(
                "ENERGY DEPLETED",
                center.x,
                center.y - context.rem_to_px(2.5),
            )
            .unwrap();

        self.level_menu.on_update(
            context,
            [center.x, center.y + context.rem_to_px(1.5)].into(),
        );
        self.retry.on_update(
            context,
            [center.x, center.y + context.rem_to_px(3.5)].into(),
        );

        StateTransition::None
    }
}
