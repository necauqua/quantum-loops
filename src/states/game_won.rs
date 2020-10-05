use crate::{engine::{
    event::Event,
    GameState,
    Context,
    StateTransition,
}, engine::util::RemConversions, QuantumLoops, states::main_game::MainGameState, engine};
use crate::ui::Button;
use crate::states::level_select::LevelMenuState;
use crate::states::main_game::TEXT_COLOR;
use crate::level::StoredData;
use crate::states::scores::ScoresState;

#[derive(Debug)]
pub struct GameWonState {
    game_state: MainGameState,
    next_level: Button,
    retry: Button,
    level_menu: Button,
    score: f32,
    best: f32,
}

impl GameWonState {
    pub fn new(game_state: MainGameState, score: f32) -> Self {
        Self {
            game_state,
            score,
            best: 0.0,
            retry: Button::new("Retry".into())
                .with_size(1.5),
            level_menu: Button::new("Level Menu".into())
                .with_size(1.5),
            next_level: Button::new("Next Level".into()),
        }
    }
}

impl GameState<QuantumLoops> for GameWonState {
    fn on_pushed(&mut self, _context: &mut Context<QuantumLoops>) {
        let mut best_scores = engine::get_data::<StoredData>().best_scores;

        let level_idx = self.game_state.level_idx();
        self.best = best_scores.get(level_idx)
            .copied()
            .unwrap_or_default();

        if self.score > self.best {
            while best_scores.len() <= level_idx {
                best_scores.push(0.0);
            }
            best_scores[level_idx] = self.score;
            engine::set_data(StoredData {
                best_scores,
                ..engine::get_data()
            });
        };
    }

    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 82, .. } = event {
            StateTransition::Set(Box::new(MainGameState::new(self.game_state.level_idx())))
        } else if self.next_level.on_event(&event, context) {
            let level_idx = self.game_state.level_idx();
            if level_idx == context.game().level_count() - 1 {
                StateTransition::Push(Box::new(ScoresState::new()))
            } else {
                engine::set_data(StoredData {
                    unlocked_level: level_idx + 1,
                    ..engine::get_data()
                });
                StateTransition::Set(Box::new(MainGameState::new(level_idx + 1)))
            }
        } else if self.level_menu.on_event(&event, context) {
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
        surface.set_fill_style(&TEXT_COLOR.into());
        surface.set_font("5rem monospace");
        let center = context.size() / 2.0;

        surface.fill_text("YOU WIN", center.x as f64, center.y as f64 - 3.5.rem_to_pixels()).unwrap();

        surface.set_font("2rem monospace");
        surface.fill_text(&format!("Efficiency: {:.2}%", self.score), center.x as f64, (center.y + 0.0.rem_to_pixels()) as f64).unwrap();

        surface.set_font("1rem monospace");
        let subtext = if self.score > self.best {
            "new best!".into()
        } else {
            format!("your best is {:.2}%", self.best)
        };

        surface.fill_text(&subtext, center.x as f64, (center.y + 1.5.rem_to_pixels()) as f64).unwrap();

        self.retry.pos = [center.x, center.y + 3.5.rem_to_pixels()].into();
        self.level_menu.pos = [center.x, center.y + 5.0.rem_to_pixels()].into();
        self.next_level.pos = [center.x, center.y + 7.0.rem_to_pixels()].into();

        self.next_level.text = (if self.game_state.level_idx() != context.game().level_count() - 1 {
            "Next Level"
        } else {
            "View the scores"
        }).into();

        self.retry.render(context);
        self.level_menu.render(context);
        self.next_level.render(context);

        StateTransition::None
    }
}
