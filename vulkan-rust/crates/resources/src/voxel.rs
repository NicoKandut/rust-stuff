use vox_format::{data::VoxModels, types::Model};

pub fn read_models(path: &str) -> VoxModels<Model> {
    vox_format::from_file(path).unwrap()
}
