use crate::{
    engine::{
        event::Event,
        GameState,
        Context,
        StateTransition,
    },
    engine::util::RemConversions,
    QuantumLoops,
    states::main_game::MainGameState,
};
use crate::ui::Button;
use crate::states::level_select::LevelMenuState;

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
            level_menu: Button::new("Level Menu".into())
                .with_size(1.5),
            retry: Button::new("Retry".into()),
        }
    }
}

impl GameState<QuantumLoops> for GameLostState {
    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 82, .. } = event {
            return StateTransition::Set(Box::new(MainGameState::new(self.game_state.level_idx())));
        }
        if self.level_menu.on_event(&event, context) {
            StateTransition::Set(Box::new(LevelMenuState::new()))
        } else if self.retry.on_event(&event, context) {
            StateTransition::Set(Box::new(MainGameState::new(self.game_state.level_idx())))
        } else {
            StateTransition::None
        }
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {

        self.game_state.on_update(context);

        let surface = context.surface();
        surface.set_fill_style(&"red".into());
        surface.set_font("5rem monospace");
        let center = context.size() / 2.0;

        surface.fill_text("ENERGY DEPLETED", center.x as f64, center.y as f64 - 2.5.rem_to_pixels()).unwrap();

        self.level_menu.pos = [center.x, center.y + 1.5.rem_to_pixels()].into();
        self.retry.pos = [center.x, center.y + 3.5.rem_to_pixels()].into();

        self.level_menu.render(context);
        self.retry.render(context);

        StateTransition::None
    }
}
