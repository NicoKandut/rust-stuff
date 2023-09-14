use std::ops::{Add, AddAssign, Sub};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorldPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
}

impl Add<i32> for &WorldPosition {
    type Output = WorldPosition;

    fn add(self, rhs: i32) -> Self::Output {
        Self::Output::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Add<usize> for &WorldPosition {
    type Output = WorldPosition;

    fn add(self, rhs: usize) -> Self::Output {
        Self::Output::new(
            self.x + rhs as i32,
            self.y + rhs as i32,
            self.z + rhs as i32,
        )
    }
}

impl Add<Self> for &WorldPosition {
    type Output = WorldPosition;

    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign<i32> for WorldPosition {
    fn add_assign(&mut self, rhs: i32) {
        self.x += rhs;
        self.y += rhs;
        self.z += rhs;
    }
}

impl Sub<i32> for &WorldPosition {
    type Output = WorldPosition;

    fn sub(self, rhs: i32) -> Self::Output {
        Self::Output::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}
