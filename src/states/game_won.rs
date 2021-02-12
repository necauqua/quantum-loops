use crate::{
    engine::{event::Event, ui::Button, Context, GameState, StateTransition},
    level::StoredData,
    states::{
        level_select::LevelMenuState,
        main_game::{MainGameState, TEXT_COLOR},
        scores::ScoresState,
    },
    QuantumLoops,
};

#[derive(Debug)]
pub struct GameWonState {
    game_state: MainGameState,
    next_level: Button,
    retry: Button,
    level_menu: Button,
    score: f64,
    best: f64,
}

impl GameWonState {
    pub fn new(game_state: MainGameState, score: f64) -> Self {
        Self {
            game_state,
            score,
            best: 0.0,
            retry: Button::new("Retry".into()).with_size(1.5),
            level_menu: Button::new("Level Menu".into()).with_size(1.5),
            next_level: Button::empty(),
        }
    }
}

impl GameState<QuantumLoops> for GameWonState {
    fn on_pushed(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let mut best_scores = context.storage().best_scores.clone();

        let level_idx = self.game_state.level_idx();
        self.best = best_scores.get(level_idx).copied().unwrap_or_default();

        if self.score > self.best {
            while best_scores.len() <= level_idx {
                best_scores.push(0.0);
            }
            best_scores[level_idx] = self.score;

            context.set_storage(StoredData {
                best_scores,
                ..context.storage().clone()
            });
        }
        StateTransition::None
    }

    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 82, .. } = event {
            StateTransition::set(MainGameState::new(self.game_state.level_idx()))
        } else if self.next_level.on_event(&event, context) {
            let level_idx = self.game_state.level_idx();
            if level_idx == context.game.level_count() - 1 {
                StateTransition::push(ScoresState::new())
            } else {
                StateTransition::set(MainGameState::new(level_idx + 1))
            }
        } else if self.level_menu.on_event(&event, context) {
            StateTransition::set(LevelMenuState::new())
        } else if self.retry.on_event(&event, context) {
            StateTransition::set(MainGameState::new(self.game_state.level_idx()))
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
            .fill_text("YOU WIN", center.x, center.y - context.rem_to_px(3.5))
            .unwrap();

        surface.set_font("2rem monospace");
        surface
            .fill_text(
                &format!("Efficiency: {:.2}%", self.score),
                center.x,
                center.y,
            )
            .unwrap();

        surface.set_font("1rem monospace");
        let subtext = if self.score > self.best {
            "new best!".into()
        } else {
            format!("your best is {:.2}%", self.best)
        };
        surface
            .fill_text(&subtext, center.x, center.y + context.rem_to_px(1.5))
            .unwrap();

        self.next_level.set_text(
            if self.game_state.level_idx() != context.game.level_count() - 1 {
                "Next Level"
            } else {
                "View the scores"
            }
            .into(),
        );

        self.retry.on_update(
            context,
            [center.x, center.y + context.rem_to_px(3.5)].into(),
        );
        self.level_menu.on_update(
            context,
            [center.x, center.y + context.rem_to_px(5.0)].into(),
        );
        self.next_level.on_update(
            context,
            [center.x, center.y + context.rem_to_px(7.0)].into(),
        );

        StateTransition::None
    }
}
