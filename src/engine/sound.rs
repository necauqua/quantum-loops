use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{*, prelude::*};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, HtmlAudioElement, Response};

use wasm_bindgen_futures::{JsFuture, spawn_local};
use crate::engine::window;

pub struct Sound {
    context: AudioContext,
    playing: Rc<RefCell<Option<AudioBufferSourceNode>>>,
    buffer: Rc<RefCell<Option<AudioBuffer>>>,
    volume: f32,
    looped: bool,
}

impl Sound {
    pub(super) fn load(context: AudioContext, url: &str) -> Self {
        let buffer = Rc::new(RefCell::new(None));

        let moved_buffer = buffer.clone();
        let moved_context = context.clone();
        let url = url.to_owned();
        spawn_local(async move {
            *moved_buffer.borrow_mut() = Some(async move {
                let response: Response = JsFuture::from(window().fetch_with_str(&url))
                    .await?
                    .dyn_into()?;
                let buffer: js_sys::ArrayBuffer = JsFuture::from(response.array_buffer()?)
                    .await?
                    .dyn_into()?;

                JsFuture::from(moved_context.decode_audio_data(&buffer)?)
                    .await?
                    .dyn_into::<AudioBuffer>()
            }.await.unwrap());
        });

        Sound {
            context,
            buffer,
            playing: Default::default(),
            volume: 1.0,
            looped: false,
        }
    }

    pub fn looped(mut self) -> Self {
        self.looped = true;
        self
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn play(&self) {
        if let Some(buffer) = self.buffer.borrow().as_ref() {
            let source = self.context.create_buffer_source().unwrap();
            source.set_buffer(Some(buffer));

            let destination = &self.context.destination();

            let gain = self.context.create_gain().unwrap();
            gain.gain().set_value(self.volume);
            gain.connect_with_audio_node(destination).unwrap();
            source.connect_with_audio_node(&gain).unwrap();

            source.set_loop(self.looped);
            source.start().unwrap();

            let moved_playing = self.playing.clone();
            source.set_onended(Some(Closure::once_into_js(move || {
                *moved_playing.borrow_mut() = None;
            }).unchecked_ref()));

            *self.playing.borrow_mut() = Some(source);
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
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
    volume: f32,
}

impl Music {
    pub(super) fn load(url: &str) -> Self {
        Music {
            audio: HtmlAudioElement::new_with_src(url).unwrap(),
            volume: 1.0,
        }
    }

    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume;
        self
    }

    pub fn looped(self) -> Self {
        self.audio.set_loop(true);
        self
    }

    pub fn playing(&self) -> bool {
        !self.audio.paused()
    }

    pub fn play(&self) {
        self.audio.set_volume(self.volume as f64);
        let _ = self.audio.play().unwrap();
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.audio.set_volume(volume as f64);
        self.volume = volume;
    }

    pub fn stop(&self) {
        self.audio.pause().unwrap();
        self.audio.set_current_time(0.0);
    }
}
