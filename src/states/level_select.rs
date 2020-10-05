use crate::{
    engine::{
        self,
        Context,
        event::Event,
        GameState,
        StateTransition,
        util::RemConversions,
    },
    level::StoredData,
    QuantumLoops,
    states::{
        main_game::MainGameState,
        main_menu::{Background, MainMenuState},
    },
    ui::Button,
};

pub struct LevelMenuState {
    levels_added: bool,
    background: Background,
    buttons: Vec<Button>,
    button_scroll: f32,
    button_limit: f32,
}

impl LevelMenuState {
    pub fn new() -> Self {
        Self {
            levels_added: false,
            background: Background::new(),
            buttons: Vec::new(),
            button_scroll: 0.0,
            button_limit: 0.0,
        }
    }
}

impl GameState<QuantumLoops> for LevelMenuState {
    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        match event {
            Event::MouseWheel { delta, .. } => {
                let yoff = -delta.y * 10.0;
                let new_scroll = self.button_scroll - yoff;

                if new_scroll >= 0.0 && new_scroll < self.button_limit {
                    self.button_scroll = new_scroll;
                    for b in &mut self.buttons {
                        b.pos.y += yoff;
                    }
                }
                StateTransition::None
            }
            _ => {
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
        }
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        // levels finally arrived
        if let Some(levels) = context.game().levels.borrow_mut().as_ref() {
            if !self.levels_added {
                self.levels_added = true;

                let size = context.size();
                let x = size.x * 0.5;
                let off = 2.5.rem_to_pixels() * 1.5;
                let mut y = size.y * 0.25;

                y += off;
                self.buttons.push(Button::new(" ← back  ".into())
                    .with_pos([x, y].into()));

                let unlocked = engine::get_data::<StoredData>().unlocked_level;

                for (idx, level) in levels.iter().enumerate() {
                    y += off;
                    let mut button = Button::new(level.name.clone().into())
                        .with_pos([x, y].into());
                    button.enabled = idx <= unlocked;
                    self.buttons.push(button);
                }

                self.button_limit = self.buttons.len() as f32 * off - size.y * 0.25;
            }
        }

        self.background.render(context);
        for button in &mut self.buttons {
            button.render(context);
        }
        StateTransition::None
    }
}
