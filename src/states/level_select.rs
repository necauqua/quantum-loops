use crate::{
    engine::{event::Event, ui::Button, Context, GameState, StateTransition},
    states::{
        main_game::MainGameState,
        main_menu::{Background, MainMenuState},
    },
    QuantumLoops,
};
use nalgebra::Vector2;

#[derive(Debug)]
pub struct LevelMenuState {
    levels_added: bool,
    background: Background,
    buttons: Vec<Button>,
    scroll: f64,
    button_limit: f64,
    last_touch: Option<Vector2<f64>>,
}

impl LevelMenuState {
    pub fn new() -> Self {
        Self {
            levels_added: false,
            background: Background::new(),
            buttons: Vec::new(),
            scroll: 0.0,
            button_limit: 0.0,
            last_touch: None,
        }
    }
}

impl GameState<QuantumLoops> for LevelMenuState {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        match &event {
            Event::KeyDown { code: 27, .. } => return StateTransition::set(MainMenuState::new()),
            Event::MouseWheel { delta, .. } => {
                let new_scroll = self.scroll + delta.y * 10.0;
                if new_scroll >= 0.0 && new_scroll < self.button_limit {
                    self.scroll = new_scroll;
                }
            }
            // Event::TouchStart { touches } if touches.len() == 1 => {
            //     self.last_touch = touches.get(0).copied();
            // }
            Event::TouchMove { touches } => {
                if let Some(touch) = touches.get(0).copied() {
                    if let Some(last_touch) = self.last_touch {
                        let delta = last_touch - touch;
                        self.last_touch = Some(touch);
                        let new_scroll = self.scroll + delta.y;
                        if new_scroll >= 0.0 && new_scroll < self.button_limit {
                            self.scroll = new_scroll;
                        }
                    }
                    self.last_touch = Some(touch);
                }
                return StateTransition::None;
            }
            Event::TouchEnd { .. } => {
                if self.last_touch.is_some() {
                    self.last_touch = None;
                    return StateTransition::None;
                }
            }
            _ => {}
        }
        for (i, button) in self.buttons.iter_mut().enumerate() {
            if button.on_event(&event, context) {
                return StateTransition::Set(if i == 0 {
                    Box::new(MainMenuState::new())
                } else {
                    Box::new(MainGameState::new(i - 1))
                });
            }
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.background.on_update(context);

        let size = context.surface().size();
        let off = context.rem_to_px(2.5) * 1.5;

        // levels finally arrived
        if let Some(levels) = context.game.levels.borrow_mut().as_ref() {
            if !self.levels_added {
                self.levels_added = true;
                self.buttons.push(Button::new(" ← back  ".into()));

                let unlocked = context.storage().unlocked_level;

                for (idx, level) in levels.iter().enumerate() {
                    let mut button = Button::new(level.name.clone().into());
                    button.enabled = idx <= unlocked;
                    self.buttons.push(button);
                }

                self.button_limit = self.buttons.len() as f64 * off - size.y * 0.25;
            }
        }

        let x = size.x * 0.5;
        let mut y = size.y * 0.25 - self.scroll;

        for button in &mut self.buttons {
            y += off;
            button.on_update(context, [x, y].into());
        }

        StateTransition::None
    }
}
