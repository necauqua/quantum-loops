use web_sys::HtmlAudioElement;

pub struct Sound {
    audio: HtmlAudioElement,
}

impl Sound {
    pub(super) fn load(url: &str) -> Sound {
        let audio = HtmlAudioElement::new().unwrap();

        audio.set_src(url);

        Sound { audio }
    }

    pub fn looped(self) -> Self {
        self.audio.set_loop(true);
        self
    }

    pub fn play(&self) {
        let _ = self.audio.play().unwrap();
    }

    pub fn stop(&self) {
        self.audio.pause().unwrap();
        self.audio.set_current_time(0.0);
    }
}
