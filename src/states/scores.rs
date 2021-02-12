use crate::engine::ui::Button;
use crate::{
    engine::{event::Event, Context, GameState, StateTransition},
    states::main_game::TEXT_COLOR,
    states::main_menu::Background,
    QuantumLoops,
};

#[derive(Debug)]
pub struct ScoresState {
    background: Background,
    back: Button,
    scroll: f64,
    limit: f64,
}

impl ScoresState {
    pub fn new() -> Self {
        Self {
            background: Background::new(),
            back: Button::new(" ← back  ".into()),
            scroll: 0.0,
            limit: 0.0,
        }
    }
}

impl GameState<QuantumLoops> for ScoresState {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        match event {
            Event::KeyDown { code: 27, .. } => StateTransition::Pop,
            Event::MouseWheel { delta, .. } => {
                let yoff = -delta.y * 10.0;
                let new_scroll = self.scroll - yoff;
                if new_scroll >= 0.0 && new_scroll < self.limit {
                    self.scroll = new_scroll;
                }
                StateTransition::None
            }
            _ => {
                if self.back.on_event(&event, context) {
                    StateTransition::Pop
                } else {
                    StateTransition::None
                }
            }
        }
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.background.on_update(context);

        let size = context.surface().size();
        let x = size.x * 0.5;
        let off = context.rem_to_px(2.5) * 1.5;
        let mut y = size.y * 0.25 - self.scroll;

        y += off;
        self.back.on_update(context, [x, y].into());

        if let Some(levels) = context.game.levels.borrow_mut().as_ref() {
            let best_scores = &context.storage().best_scores;
            let surface = context.surface().context();

            for (idx, level) in levels.iter().enumerate() {
                y += off;

                let best = best_scores.get(idx).copied().unwrap_or_default();

                surface.set_fill_style(&TEXT_COLOR.into());
                surface.set_font("2.5rem monospace");
                surface
                    .fill_text(&format!("{}: {:.2}%", level.name, best), x, y)
                    .unwrap();
            }
            self.limit = levels.len() as f64 * off - size.y * 0.25;
        }

        StateTransition::None
    }
}
