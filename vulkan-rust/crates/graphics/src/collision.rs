pub trait CollisionDetection<T> {
    fn collides_with(&self, other: T) -> bool;
}
