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

    pub fn distance_squared(a: &Self, b: &Self) -> i32 {
        let distance_x = (b.x - a.x).pow(2);
        let distance_y = (b.y - a.y).pow(2);
        let distance_z = (b.z - a.z).pow(2);

        distance_x + distance_y + distance_z
    }

    pub fn rem_euclid(&self, n: i32) -> Self {
        let x = self.x.rem_euclid(n);
        let y = self.y.rem_euclid(n);
        let z = self.z.rem_euclid(n);

        assert!(x >= 0);
        assert!(y >= 0);
        assert!(z >= 0);
        assert!(x < n);
        assert!(y < n);
        assert!(z < n);

        Self::new(x, y, z)
    }
}

impl<T> Add<T> for WorldPosition
where
    T: Into<i32>,
{
    type Output = WorldPosition;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl<T> Add<T> for &WorldPosition
where
    T: Into<i32>,
{
    type Output = WorldPosition;
    fn add(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl Add<Self> for WorldPosition {
    type Output = WorldPosition;
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl Add<Self> for &WorldPosition {
    type Output = WorldPosition;
    fn add(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T> AddAssign<T> for WorldPosition
where
    T: Into<i32>,
{
    fn add_assign(&mut self, rhs: T) {
        let value = rhs.into();
        self.x += value;
        self.y += value;
        self.z += value;
    }
}

impl<T> Sub<T> for WorldPosition
where
    T: Into<i32>,
{
    type Output = WorldPosition;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl<T> Sub<T> for &WorldPosition
where
    T: Into<i32>,
{
    type Output = WorldPosition;
    fn sub(self, rhs: T) -> Self::Output {
        let rhs = rhs.into();
        Self::Output::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl Sub<Self> for WorldPosition {
    type Output = WorldPosition;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Sub<Self> for &WorldPosition {
    type Output = WorldPosition;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::Output::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl From<&glm::Vec3> for WorldPosition {
    fn from(value: &glm::Vec3) -> Self {
        Self::new(
            value.x.floor() as i32,
            value.y.floor() as i32,
            value.z.floor() as i32,
        )
    }
}

impl From<&WorldPosition> for glm::Vec3 {
    fn from(value: &WorldPosition) -> Self {
        Self::new(value.x as f32, value.y as f32, value.z as f32)
    }
}
