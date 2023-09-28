use crate::{read_image, read_models};
use png::OutputInfo;
use std::collections::HashMap;
use vox_format::{data::VoxModels, types::Model};

pub struct Cache {
    vox: HashMap<String, VoxModels<Model>>,
    img: HashMap<String, (OutputInfo, Vec<u8>)>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self {
            vox: load_all_vox_models(),
            img: load_all_images(),
        }
    }

    pub fn get_vox(&self, path: &str) -> &VoxModels<Model> {
        self.vox.get(path).unwrap()
    }

    pub fn get_img(&self, path: &str) -> &(OutputInfo, Vec<u8>) {
        self.img.get(path).unwrap()
    }
}

fn load_all_vox_models() -> HashMap<String, VoxModels<Model>> {
    let vox_paths = ["D:/Projects/rust-stuff/vulkan-rust/assets/tree.vox"];
    let mut vox = HashMap::with_capacity(vox_paths.len());
    for path in vox_paths {
        vox.insert(path.to_owned(), read_models(path));
    }

    vox
}

fn load_all_images() -> HashMap<String, (OutputInfo, Vec<u8>)> {
    let img_paths = [
        "D:/Projects/rust-stuff/vulkan-rust/assets/palette.png",
        "D:/Projects/rust-stuff/vulkan-rust/assets/tileset.png",
    ];
    let mut img = HashMap::with_capacity(img_paths.len());
    for path in img_paths {
        img.insert(path.to_owned(), read_image(path).unwrap());
    }

    img
}
