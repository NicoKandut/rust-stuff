// type composition for vectors and matricies
use std::f64::consts::PI;
use std::ops::Add;
use std::ops::Mul;

pub type V2<T> = [T; 2];
pub type V3<T> = [T; 3];
pub type V4<T> = [T; 4];
pub type M2x2<T> = [V2<T>; 2];
pub type M3x3<T> = [V3<T>; 3];
pub type M4x4<T> = [V4<T>; 4];

pub trait Dot {
  type Output;
  fn dot(self, rhs: Self) -> Self::Output;
}

impl<T> Dot for V4<T>
where
  T: Add + Mul,
{
  type Output = T;

  fn dot(self, rhs: Self) -> Self::Output {
    let a = self[0] * rhs[0];
    return self[0] * rhs[0] + self[1] * rhs[1] + self[2] * rhs[2] + self[3] * rhs[3];
  }
}

pub trait Cross<Rhs = Self> {
  type Output;

  fn cross(self, rhs: Rhs) -> Self::Output;
}

impl Cross<V4<f64>> for M4x4<f64> {
  type Output = V4<f64>;

  fn cross(self, rhs: V4<f64>) -> V4<f64> {
    return [
      self[0][0] * rhs[0] + self[1][0] * rhs[1] + self[2][0] * rhs[2] + self[3][0] * rhs[3],
      self[0][1] * rhs[0] + self[1][1] * rhs[1] + self[2][1] * rhs[2] + self[3][1] * rhs[3],
      self[0][2] * rhs[0] + self[1][2] * rhs[1] + self[2][2] * rhs[2] + self[3][2] * rhs[3],
      self[0][3] * rhs[0] + self[1][3] * rhs[1] + self[2][3] * rhs[2] + self[3][3] * rhs[3],
    ];
  }
}

pub fn get_perspective_matrices(
  vertical_fov: f64,
  aspect_ratio: f64,
  n: f64,
  f: f64,
) -> (M4x4<f64>, M4x4<f64>) {
  let fov_rad = vertical_fov * 2.0 * PI / 360.0;
  let focal_length = 1.0 / (fov_rad / 2.0).tan();

  let x = focal_length / aspect_ratio;
  let y = -focal_length;
  let a = n / (f - n);
  let b = f * a;

  let projection: M4x4<f64> = [
    [x, 0.0, 0.0, 0.0],
    [0.0, y, 0.0, 0.0],
    [0.0, 0.0, a, b],
    [0.0, 0.0, -1.0, 0.0],
  ];

  let inverse: M4x4<f64> = [
    [1.0 / x, 0.0, 0.0, 0.0],
    [0.0, 1.0 / y, 0.0, 0.0],
    [0.0, 0.0, 0.0, -1.0],
    [0.0, 0.0, 1.0 / b, a / b],
  ];

  return (projection, inverse);
}
