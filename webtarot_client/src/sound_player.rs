use std::collections::HashMap;
use web_sys::HtmlAudioElement;
// use js_sys;

pub struct SoundPlayer {
    sounds: HashMap<String, HtmlAudioElement>
}

impl SoundPlayer {
    pub fn new(sound_paths: HashMap<String, &str>) -> Self {
        let mut sounds = HashMap::new();
        for (k, p) in sound_paths {
            if let Ok(sound) = HtmlAudioElement::new_with_src(p) {
                sounds.insert(k, sound);
            }
        }
        Self { sounds }
    }

    // pub fn play(self, slug: &str) -> Result<js_sys::Promise, wasm_bindgen::JsValue> {
    pub fn play(&self, slug: &str) {
        if let Some(sound) = self.sounds.get(slug) {
            let _res = sound.play();
        }
    }
}

























                                             
                                             
                                             
                                             
                                             
                                             
                                             
