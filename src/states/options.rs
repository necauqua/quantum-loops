use nalgebra::Vector2;

use crate::{
    engine::{event::Event, ui::Button, Context, GameState, StateTransition},
    level::StoredData,
    states::{
        main_menu::{Background, MainMenuState},
        tutorial::TutorialState,
    },
    QuantumLoops,
};

#[derive(Debug)]
pub struct OptionsState {
    background: Background,
    back: Button,
    reset: Button,
    tutorial: Button,
    sounds: Button,
    music: Button,
    sure_timer: f64,
}

impl OptionsState {
    pub fn new() -> Self {
        Self {
            background: Background::new(),
            back: Button::new(" ← back  ".into()),
            reset: Button::empty(),
            tutorial: Button::new("Open the tutorial".into()),
            sounds: Button::empty(),
            music: Button::empty(),
            sure_timer: 0.0,
        }
    }
}

impl GameState<QuantumLoops> for OptionsState {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 27, .. } = event {
            return StateTransition::set(MainMenuState::new());
        }
        if self.reset.on_event(&event, context) {
            if self.sure_timer <= 0.0 {
                self.sure_timer = 3.0;
            } else {
                context.set_storage(Default::default());
                return StateTransition::set(MainMenuState::new());
            }
        } else if self.tutorial.on_event(&event, context) {
            return StateTransition::set(TutorialState::new());
        } else if self.sounds.on_event(&event, context) {
            let data = context.storage().clone();
            context.sound_context_mut().sound_mask.set(0, !data.sounds_enabled);
            context.set_storage(StoredData {
                sounds_enabled: !data.sounds_enabled,
                ..data
            });
        } else if self.music.on_event(&event, context) {
            let data = context.storage().clone();
            context.sound_context_mut().sound_mask.set(1, !data.music_enabled);
            let bg = &context.game.sounds.background;
            if data.music_enabled {
                bg.stop();
            } else {
                bg.play_unique();
            }
            context.set_storage(StoredData {
                music_enabled: !data.music_enabled,
                ..data
            });
        } else if self.back.on_event(&event, context) {
            return StateTransition::set(MainMenuState::new());
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let center = context.surface().size() / 2.0;
        let offset: Vector2<f64> = [0.0, context.rem_to_px(2.5)].into();

        if self.sure_timer > 0.0 {
            self.sure_timer -= context.delta_time();
        }

        self.background.on_update(context);

        let s = context.storage();

        self.reset.set_text(
            if self.sure_timer <= 0.0 {
                "Full reset"
            } else {
                "Full reset (are you sure?)"
            }
            .into(),
        );
        self.sounds.set_text(
            if s.sounds_enabled {
                "Sounds: On"
            } else {
                "Sounds: Off"
            }
            .into(),
        );
        self.music.set_text(
            if s.music_enabled {
                "Music: On"
            } else {
                "Music: Off"
            }
            .into(),
        );

        self.back.on_update(context, center - offset * 2.0);
        self.reset.on_update(context, center - offset);
        self.tutorial.on_update(context, center);
        self.sounds.on_update(context, center + offset);
        self.music.on_update(context, center + offset * 2.0);

        StateTransition::None
    }
}
