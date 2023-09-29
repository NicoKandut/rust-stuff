use crate::{read_image, read_models};
use logging::{log, LOG_CACHE};
use png::OutputInfo;
use std::{collections::HashMap, env};
use vox_format::{data::VoxModels, types::Model};

pub struct Cache {
    vox: HashMap<String, VoxModels<Model>>,
    img: HashMap<String, (OutputInfo, Vec<u8>)>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        let cache = Self {
            vox: load_all_vox_models(),
            img: load_all_images(),
        };

        log!(
            *LOG_CACHE,
            " Created cache with {} vox models and {} images",
            cache.vox.len(),
            cache.img.len()
        );

        cache
    }

    pub fn get_vox(&self, path: &str) -> &VoxModels<Model> {
        self.vox.get(path).expect("Voxel resource is required")
    }

    pub fn get_img(&self, path: &str) -> &(OutputInfo, Vec<u8>) {
        self.img.get(path).expect("Image resource is required")
    }
}

fn load_all_vox_models() -> HashMap<String, VoxModels<Model>> {
    let vox_paths = ["\\assets\\tree.vox"];
    let working_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();
    let mut vox = HashMap::with_capacity(vox_paths.len());
    for path in vox_paths {
        let abs_path = working_dir.clone() + path;
        vox.insert(path.to_owned(), read_models(&abs_path));
    }

    vox
}

fn load_all_images() -> HashMap<String, (OutputInfo, Vec<u8>)> {
    let img_paths = ["\\assets\\palette.png", "\\assets\\tileset.png"];
    let mut img = HashMap::with_capacity(img_paths.len());
    let working_dir = env::current_dir().unwrap().to_str().unwrap().to_owned();
    for path in img_paths {
        let abs_path = working_dir.clone() + path;
        img.insert(path.to_owned(), read_image(&abs_path).unwrap());
    }

    img
}
