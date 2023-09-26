mod cache;
mod image;
mod voxel;

pub use image::*;
pub use voxel::*;

pub mod prelude {
    use crate::cache::Cache;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref CACHE: Cache = Cache::new();
    }
}

#[cfg(test)]
mod tests {}
