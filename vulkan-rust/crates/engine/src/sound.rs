use std::{collections::HashMap, env};

use rfmod::Sound;

pub(crate) struct SoundEngine {
    fmod: rfmod::Sys,
    sounds: HashMap<String, Sound>,
}

impl SoundEngine {
    pub(crate) fn new() -> Self {
        let fmod = rfmod::Sys::new().expect("Fmod must be available");
        fmod.init();

        let sounds = load_sounds(&fmod);

        Self { fmod, sounds }
    }
}

fn load_sounds(fmod: &rfmod::Sys) -> HashMap<String, Sound> {
    let sound_files = ["\\assets\\break.ogg"];
    let mut sounds = HashMap::with_capacity(sound_files.len());
    let working_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();
    for file in sound_files {
        let abs_path = working_dir.clone() + file;
        let sound = fmod
            .create_sound(&abs_path, None, None)
            .expect("Sound file must be available");
        sounds.insert(file.to_owned(), sound);
    }
    sounds
}
