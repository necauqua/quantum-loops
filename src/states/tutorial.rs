use crate::{
    engine::{
        event::Event,
        GameState,
        Context,
        StateTransition,
    },
    engine,
    engine::util::RemConversions,
    level::StoredData,
    QuantumLoops,
    states::{
        main_menu::MainMenuState,
    },
    states::main_game::MainGameState,
    states::main_game::TEXT_COLOR,
    states::main_menu::Background,
    ui::Button
};

const FONT_SIZE: f64 = 2.0;

const TUTORIAL: &[&str] = &[
    "There is a particle STUCK in a quantum loop\n\n",

    "It really really hates existing, but the\n\
     quantum level with the most energy keeps it in place",

    "Your objective is to disrupt these levels such that the\n\
     particle can escape back into nothing",

    "You can only disrupt the level when it uses most of\n\
     its energy to keep the particle on it",

    "You have a limited amount of energy, it's usage depends\n\
     on the time and the length of the disruption",

    "You need to spend at least the base energy\n\
     of the level to disrupt it",

    "The more efficient you are, the better\n\n",

    "Tip for the last level:\nYou can press R to restart",
];

#[derive(Debug)]
pub struct TutorialState {
    background: Background,
    current_page: usize,
    back: Button,
    next: Button,
    skip: Button,
}

impl TutorialState {
    pub fn new() -> Self {
        Self {
            background: Background::new(),
            current_page: 0,
            back: Button::new(" ← back  ".into()),
            next: Button::new("  next → ".into()),
            skip: Button::new("skip".into())
                .with_size(1.0),
        }
    }
}

impl TutorialState {

    fn play(&mut self) -> StateTransition<QuantumLoops> {
        engine::set_data(StoredData {
            passed_tutorial: true,
            ..engine::get_data()
        });
        StateTransition::Set(Box::new(MainGameState::new(0)))
    }

    fn back(&mut self) -> Option<StateTransition<QuantumLoops>> {
        if self.current_page == 0 {
            Some(StateTransition::Set(Box::new(MainMenuState::new())))
        } else {
            self.current_page -= 1;
            None
        }
    }
}

impl GameState<QuantumLoops> for TutorialState {
    fn on_event(&mut self, event: Event, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        if let Event::KeyDown { code: 27, .. } = event {
            if let Some(transition) = self.back() {
                return transition;
            }
        }
        if self.back.on_event(&event, context) {
            if let Some(transition) = self.back() {
                return transition;
            }
        }
        if self.current_page != TUTORIAL.len() - 1 && self.skip.on_event(&event, context) {
            return self.play();
        }
        if self.next.on_event(&event, context) {
            if self.current_page == TUTORIAL.len() - 1 {
                return self.play();
            }
            self.current_page += 1;
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.background.render(context);

        let surface = context.surface();
        let center = context.size() / 2.0;

        let lines = TUTORIAL[self.current_page].lines().collect::<Vec<_>>();

        let offset = FONT_SIZE.rem_to_pixels();
        let height = (lines.len() as f64 + 1.5) * offset;
        let mut start = center.y as f64 - height / 2.0;

        surface.set_font(&format!("{}rem monospace", FONT_SIZE));
        surface.set_fill_style(&TEXT_COLOR.into());

        for line in lines {
            surface.fill_text(line, center.x as f64, start).unwrap();
            start += offset
        }

        start += offset;

        self.back.pos = [center.x - self.back.compute_size(surface).0, start as f32].into();
        self.next.pos = [center.x + self.next.compute_size(surface).0, start as f32].into();

        self.skip.pos = [center.x, (start + offset) as f32].into();

        self.next.text = (if self.current_page == TUTORIAL.len() - 1 {
            "  play → "
        } else {
            self.skip.render(context);
            "  next → "
        }).into();

        self.back.render(context);
        self.next.render(context);

        // update buttons
        StateTransition::None
    }
}
