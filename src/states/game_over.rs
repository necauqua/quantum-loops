use crate::{
    engine::{
        event::Event,
        GameState,
        GameUpdate,
        StateTransition,
    },
    engine::util::RemConversions,
    level::GameLevel,
    QuantumLoops,
    states::{
        main_game::{MainGameState, TEXT_COLOR},
        main_menu::MainMenuState,
    },
};

#[derive(Debug)]
pub struct GameOverState {
    title: String,
    color: String,
    retry_level: GameLevel,
}

impl GameOverState {
    pub fn new(title: String, color: String, retry_level: GameLevel) -> Self {
        Self {
            title,
            color,
            retry_level,
        }
    }
}

impl GameState<QuantumLoops> for GameOverState {
    fn on_mounted(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        let surface = context.surface();
        surface.set_text_align("center");
        surface.set_text_baseline("middle");
        surface.set_fill_style(&(&self.color).into());
        surface.set_font("5rem monospace");
        let center = context.size() / 2.0;
        surface.fill_text(&self.title, center.x as f64, center.y as f64 - 2.5.rem_to_pixels()).unwrap();
        surface.set_font("1rem monospace");
        surface.fill_text("Esc to go back to main menu", center.x as f64, center.y as f64 + 0.5.rem_to_pixels()).unwrap();
        surface.fill_text("Space to retry", center.x as f64, center.y as f64 + 1.5.rem_to_pixels()).unwrap();
    }

    fn on_event(&mut self, event: Event, _: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match event {
            Event::KeyDown { code: 27, .. } =>
                StateTransition::Set(Box::new(MainMenuState::new())),
            Event::KeyDown { code: 32, .. } =>
                StateTransition::Set(Box::new(MainGameState::new(self.retry_level.clone()))),
            _ => StateTransition::None,
        }
    }
}
