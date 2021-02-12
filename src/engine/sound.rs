use std::fmt::{Debug, Formatter};

use wasm_bindgen::{prelude::*, *};
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{AudioBuffer, AudioBufferSourceNode, AudioContext, Response};

use crate::engine::util::{Bitmap, Mut};
use crate::engine::window;

pub struct SoundContext {
    web_audio: AudioContext,
    pub sound_mask: Bitmap,
}

impl SoundContext {
    pub fn new() -> SoundContext {
        SoundContext {
            web_audio: AudioContext::new().unwrap(),
            sound_mask: Bitmap::full(),
        }
    }
}

pub struct Sound {
    context: Mut<SoundContext>,
    playing: Mut<Option<AudioBufferSourceNode>>,
    buffer: Mut<Option<AudioBuffer>>,
    layer_mask: Bitmap,
    volume: f64,
    looped: bool,
}

impl Debug for Sound {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Sound")
            .field("is_playing", &self.playing.borrow().is_some())
            .field("is_loaded", &self.buffer.borrow().is_some())
            .field("sound_layer_mask", &self.layer_mask)
            .field("volume", &self.volume)
            .field("looped", &self.looped)
            .finish()
    }
}

impl Sound {
    pub(super) fn load(context: Mut<SoundContext>, url: &str) -> Self {
        let buffer = Mut::new(None);

        let moved_buffer = buffer.clone();
        let moved_context = context.clone();
        let url = url.to_owned();
        spawn_local(async move {
            *moved_buffer.borrow_mut() = Some(
                async move {
                    let response: Response = JsFuture::from(window().fetch_with_str(&url))
                        .await?
                        .dyn_into()?;
                    let buffer: js_sys::ArrayBuffer =
                        JsFuture::from(response.array_buffer()?).await?.dyn_into()?;

                    JsFuture::from(
                        moved_context
                            .borrow()
                            .web_audio
                            .decode_audio_data(&buffer)?,
                    )
                    .await?
                    .dyn_into::<AudioBuffer>()
                }
                .await
                .unwrap(),
            );
        });

        Sound {
            context,
            buffer,
            playing: Default::default(),
            layer_mask: Bitmap::empty().with_on(0), // or just Bitmap::new(1)
            volume: 1.0,
            looped: false,
        }
    }

    pub fn looped(mut self) -> Self {
        self.looped = true;
        self
    }

    pub fn with_layers(mut self, mask: Bitmap) -> Self {
        self.layer_mask = mask;
        self
    }

    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume = volume;
        self
    }

    fn can_play(&self) -> bool {
        self.context.borrow().sound_mask.intersects(self.layer_mask)
    }

    pub fn play_unique(&self) {
        if !self.can_play() {
            self.stop();
        } else if !self.playing() {
            self.play()
        }
    }

    pub fn play(&self) {
        if !self.can_play() {
            self.stop();
            return;
        }
        if let Some(buffer) = self.buffer.borrow().as_ref() {
            let web_audio = &self.context.borrow().web_audio;

            let source = web_audio.create_buffer_source().unwrap();
            source.set_buffer(Some(buffer));

            let destination = &web_audio.destination();

            let gain = web_audio.create_gain().unwrap();
            gain.gain().set_value(self.volume as f32);
            gain.connect_with_audio_node(destination).unwrap();
            source.connect_with_audio_node(&gain).unwrap();

            source.set_loop(self.looped);
            source.start().unwrap();

            let moved_playing = self.playing.clone();
            source.set_onended(Some(
                Closure::once_into_js(move || {
                    *moved_playing.borrow_mut() = None;
                })
                .unchecked_ref(),
            ));

            *self.playing.borrow_mut() = Some(source);
        }
    }

    pub fn set_volume(&mut self, volume: f64) {
        self.volume = volume;
    }

    pub fn playing(&self) -> bool {
        self.playing.borrow().is_some()
    }

    pub fn stop(&self) {
        if let Some(playing) = self.playing.borrow_mut().take() {
            playing.stop().unwrap();
        }
    }
}
