use crate::{
    engine::{event::Event, ui::Button, Context, GameState, StateTransition},
    level::StoredData,
    states::{
        main_game::{MainGameState, TEXT_COLOR},
        main_menu::{Background, MainMenuState},
    },
    QuantumLoops,
};

const TUTORIAL: &[&str] = &[
    "There is a particle STUCK in a quantum loop\n\n",
    //
    "It really really hates existing, but the\n\
     quantum level with the most energy keeps it in place",
    //
    "Your objective is to disrupt these levels such that the\n\
     particle can escape back into nothing",
    //
    "You can only disrupt the level when it uses most of\n\
     its energy to keep the particle on it",
    //
    "You have a limited amount of energy, it's usage depends\n\
     on the time and the length of the disruption",
    //
    "You need to spend at least the base energy\n\
     of the level to disrupt it",
    //
    "The more efficient you are, the better\n\n",
    //
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
            next: Button::empty(),
            skip: Button::new("skip".into()).with_size(1.0),
        }
    }
}

impl TutorialState {
    fn play(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        context.set_storage(StoredData {
            passed_tutorial: true,
            ..context.storage().clone()
        });
        StateTransition::set(MainGameState::new(0))
    }

    fn back(&mut self) -> Option<StateTransition<QuantumLoops>> {
        if self.current_page == 0 {
            Some(StateTransition::set(MainMenuState::new()))
        } else {
            self.current_page -= 1;
            None
        }
    }
}

impl GameState<QuantumLoops> for TutorialState {
    fn on_event(
        &mut self,
        event: Event,
        context: &mut Context<QuantumLoops>,
    ) -> StateTransition<QuantumLoops> {
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
            return self.play(context);
        }
        if self.next.on_event(&event, context) {
            if self.current_page == TUTORIAL.len() - 1 {
                return self.play(context);
            }
            self.current_page += 1;
        }
        StateTransition::None
    }

    fn on_update(&mut self, context: &mut Context<QuantumLoops>) -> StateTransition<QuantumLoops> {
        self.background.on_update(context);

        let center = context.surface().size() / 2.0;
        let surface = context.surface().context();

        let lines = TUTORIAL[self.current_page].lines().collect::<Vec<_>>();

        let offset = context.rem_to_px(2.0);
        let height = (lines.len() as f64 + 1.5) * offset;
        let mut start = center.y - height / 2.0;

        surface.set_font("2rem monospace");
        surface.set_fill_style(&TEXT_COLOR.into());

        for line in lines {
            surface.fill_text(line, center.x, start).unwrap();
            start += offset
        }

        start += offset;

        let back_offset = self.back.text.compute_size(context).0;
        let next_offset = self.next.text.compute_size(context).0;

        self.next.set_text(
            if self.current_page == TUTORIAL.len() - 1 {
                "  play → "
            } else {
                "  next → "
            }
            .into(),
        );

        if self.current_page != TUTORIAL.len() - 1 {
            self.skip
                .on_update(context, [center.x, (start + offset)].into());
        }

        self.back
            .on_update(context, [center.x - back_offset, start].into());
        self.next
            .on_update(context, [center.x + next_offset, start].into());

        // update buttons
        StateTransition::None
    }
}
