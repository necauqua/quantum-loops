use nalgebra::Vector2;

use crate::{
    engine::{
        Context,
        event::Event,
        GameState,
        StateTransition,
    },
    engine,
    engine::util::RemConversions,
    level::StoredData,
    QuantumLoops,
    states::main_menu::{Background, MainMenuState},
    states::tutorial::TutorialState,
    ui::Button,
};

#[derive(Debug)]
pub struct OptionsState {
    background: Background,
    reset: Button,
    tutorial: Button,
    sounds: Button,
    music: Button,
    back: Button,
    sure_timer: f64,
}

impl OptionsState {
    pub fn new() -> Self {
        Self {
            background: Background::new(),
            reset: Button::new("Full reset".into()),
            tutorial: Button::new("Open the tutorial".into()),
            sounds: Button::new("Sounds: On".into()),
            music: Button::new("Music: On".into()),
            back: Button::new(" ← back  ".into()),
            sure_timer: 0.0,
        }
    }
}

impl GameState<QuantumLoops> for OptionsState {
    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 27, .. } = event {
            return StateTransition::Set(Box::new(MainMenuState::new()));
        }
        if self.reset.on_event(&event, context) {
            if self.sure_timer <= 0.0 {
                self.sure_timer = 3.0;
            } else {
                engine::set_data::<StoredData>(Default::default());
                return StateTransition::Set(Box::new(MainMenuState::new()));
            }
        } else if self.tutorial.on_event(&event, context) {
            return StateTransition::Set(Box::new(TutorialState::new()));
        } else if self.sounds.on_event(&event, context) {
            let data: StoredData = engine::get_data();
            engine::set_data(StoredData {
                sounds_enabled: !data.sounds_enabled,
                ..data
            });
        } else if self.music.on_event(&event, context) {
            let data: StoredData = engine::get_data();
            let bg = &context.game().sounds.background;
            if data.music_enabled {
                bg.stop();
            } else {
                bg.play();
            }
            engine::set_data(StoredData {
                music_enabled: !data.music_enabled,
                ..data
            });
        } else if self.back.on_event(&event, context) {
            return StateTransition::Set(Box::new(MainMenuState::new()));
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        let center = context.size() / 2.0;
        let offset: Vector2<f32> = [0.0, 2.5.rem_to_pixels()].into();

        if self.sure_timer <= 0.0 {
            self.reset.text = "Full reset".into();
        } else {
            self.sure_timer -= context.delta_time();
            self.reset.text = "Full reset (are you sure?)".into();
        }

        // this all is so dumb

        let stored_data: StoredData = engine::get_data();
        self.sounds.text = (if stored_data.sounds_enabled {
            "Sounds: On"
        } else {
            "Sounds: Off"
        }).into();

        self.music.text = (if stored_data.music_enabled {
            "Music: On"
        } else {
            "Music: Off"
        }).into();

        self.reset.pos = center - offset * 2.0;
        self.tutorial.pos = center - offset;
        self.sounds.pos = center;
        self.music.pos = center + offset;
        self.back.pos = center + offset * 2.0;

        self.background.render(context);
        self.reset.render(context);
        self.tutorial.render(context);
        self.sounds.render(context);
        self.music.render(context);
        self.back.render(context);

        StateTransition::None
    }
}
