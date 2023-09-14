pub const CHUNK_SIZE: isize = 64;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl Position {
    pub fn new(x: isize, y: isize, z: isize) -> Self {
        Self { x, y, z }
    }

    pub fn rounded_to(&self, size: isize) -> Self {
        Self::new(
            (self.x as f32 / size as f32).floor() as isize * size,
            (self.y as f32 / size as f32).floor() as isize * size,
            (self.z as f32 / size as f32).floor() as isize * size,
        )
    }

    pub fn relative_to(&self, size: isize) -> Self {
        Self::new(self.x % size, self.y % size, self.z % size)
    }

    pub fn to_child_index(&self, size: isize) -> usize {
        let x_up = self.x >= (size / 2);
        let y_up = self.y >= (size / 2);
        let z_up = self.z >= (size / 2);

        (z_up as usize) << 2 | (y_up as usize) << 1 | (x_up as usize)
    }
}

#[cfg(test)]
mod tests {
    use crate::position::CHUNK_SIZE;

    use super::Position;

    #[test]
    fn rounded_to_works() {
        assert_eq!(
            Position::new(0, 0, 0).rounded_to(CHUNK_SIZE),
            Position::new(0, 0, 0)
        );
        assert_eq!(
            Position::new(63, 63, 63).rounded_to(CHUNK_SIZE),
            Position::new(0, 0, 0)
        );
        assert_eq!(
            Position::new(0, 67, 80).rounded_to(CHUNK_SIZE),
            Position::new(0, 64, 64)
        );
        assert_eq!(
            Position::new(64, 0, 64).rounded_to(CHUNK_SIZE),
            Position::new(64, 0, 64)
        );
        assert_eq!(
            Position::new(64, 64, 0).rounded_to(CHUNK_SIZE),
            Position::new(64, 64, 0)
        );
        assert_eq!(
            Position::new(64, 64, 64).rounded_to(CHUNK_SIZE),
            Position::new(64, 64, 64)
        );
        assert_eq!(
            Position::new(0, 0, -1).rounded_to(CHUNK_SIZE),
            Position::new(0, 0, -64)
        );
        assert_eq!(
            Position::new(0, -1, 0).rounded_to(CHUNK_SIZE),
            Position::new(0, -64, 0)
        );
        assert_eq!(
            Position::new(-1, 0, 0).rounded_to(CHUNK_SIZE),
            Position::new(-64, 0, 0)
        );
        assert_eq!(
            Position::new(0, 0, -64).rounded_to(CHUNK_SIZE),
            Position::new(0, 0, -64)
        );
        assert_eq!(
            Position::new(0, -64, 0).rounded_to(CHUNK_SIZE),
            Position::new(0, -64, 0)
        );
        assert_eq!(
            Position::new(-64, 0, 0).rounded_to(CHUNK_SIZE),
            Position::new(-64, 0, 0)
        );
    }
}
