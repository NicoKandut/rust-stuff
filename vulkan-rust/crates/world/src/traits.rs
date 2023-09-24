pub trait Generate<S> {
    fn generate(seed: S) -> Self;
}

pub trait Voxelize<T> {
    fn voxelize(&self) -> T;
}

pub trait Data3D<T> {
    fn set(&mut self, x: usize, y: usize, z: usize, value: T);
    fn get(&self, x: usize, y: usize, z: usize) -> T;
}
