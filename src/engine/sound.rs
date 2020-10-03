use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{*, prelude::*};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, HtmlAudioElement};

use super::util::PromiseGlue;

#[wasm_bindgen(inline_js="\
export function load_audio_buffer(ctx, url) {
    return fetch(url)
      .then(res => res.arrayBuffer())
      .then((buffer) => {
        return new Promise((resolve, reject) => {
          ctx.decodeAudioData(buffer, (audioBuffer) => {
            resolve(audioBuffer);
          });
        });
      });
}")]
extern "C" {
    fn load_audio_buffer(context: &AudioContext, url: &str) -> js_sys::Promise;
}

pub struct Sound {
    context: AudioContext,
    playing: Rc<RefCell<Option<AudioBufferSourceNode>>>,
    buffer: Rc<RefCell<Option<AudioBuffer>>>,
    looped: bool,
}

impl Sound {
    pub(super) fn load(context: AudioContext, url: &str) -> Self {
        let buffer = Rc::new(RefCell::new(None));
        let moved_buffer = buffer.clone();

        let _ = load_audio_buffer(&context, url)
            .rust_then(move |res| {
                *moved_buffer.borrow_mut() = Some(res);
            });

        Sound {
            context,
            buffer,
            playing: Rc::new(RefCell::new(None)),
            looped: false,
        }
    }

    pub fn looped(mut self) -> Self {
        self.looped = true;
        self
    }

    pub fn play(&mut self) {
        if let Some(buffer) = self.buffer.borrow().as_ref() {
            let source = self.context.create_buffer_source().unwrap();
            source.set_buffer(Some(buffer));
            source.connect_with_audio_node(&self.context.destination()).unwrap();
            source.set_loop(self.looped);
            source.start().unwrap();

            let moved_playing = self.playing.clone();
            source.set_onended(Some(Closure::once_into_js(move || {
                *moved_playing.borrow_mut() = None;
            }).unchecked_ref()));

            *self.playing.borrow_mut() = Some(source);
        }
    }

    pub fn playing(&self) -> bool {
        self.playing.borrow().is_some()
    }

    pub fn stop(&mut self) {
        if let Some(playing) = self.playing.borrow_mut().take() {
            playing.stop().unwrap();
        }
    }
}

pub struct Music {
    audio: HtmlAudioElement,
}

impl Music {
    pub(super) fn load(url: &str) -> Self {
        Music {
            audio: HtmlAudioElement::new_with_src(url).unwrap()
        }
    }

    pub fn looped(self) -> Self {
        self.audio.set_loop(true);
        self
    }

    pub fn playing(&self) -> bool {
        !self.audio.paused()
    }

    pub fn play(&self) {
        let _ = self.audio.play().unwrap();
    }

    pub fn set_volume(&self, volume: f64) {
        self.audio.set_volume(volume);
    }

    pub fn stop(&self) {
        self.audio.pause().unwrap();
        self.audio.set_current_time(0.0);
    }
}
