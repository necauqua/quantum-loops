use crate::{
    engine::{
        event::{Event, MouseButton},
        GameState,
        GameUpdate,
        StateTransition,
    },
    QuantumLoops,
    states::main_game::TEXT_COLOR
};

#[derive(Debug)]
pub struct PauseState;

impl PauseState {
    pub fn new() -> Self {
        Self
    }
}

impl GameState<QuantumLoops> for PauseState {
    fn on_mounted(&mut self, context: &mut GameUpdate<QuantumLoops>) {
        let surface = context.surface();
        surface.set_text_align("center");
        surface.set_text_baseline("middle");
        surface.set_fill_style(&TEXT_COLOR.into());
        surface.set_font("5rem monospace");
        let center = context.size() / 2.0;
        surface.fill_text("PAUSED", center.x as f64, center.y as f64).unwrap();
    }

    fn on_event(&mut self, event: Event, _: &mut GameUpdate<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match event {
            Event::KeyDown { code: 27, .. } |
            Event::MouseDown { button: MouseButton::Left, .. } => StateTransition::Pop,
            _ => StateTransition::None,
        }
    }
}
