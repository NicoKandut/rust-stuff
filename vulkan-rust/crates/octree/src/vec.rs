use crate::octree::{Point, Size};

pub(crate) fn mul_s(v: Point, s: Size) -> Point {
  [v[0] * s, v[1] * s, v[2] * s]
}

pub(crate) fn div_s(v: Point, s: Size) -> Point {
  [v[0] / s, v[1] / s, v[2] / s]
}

pub(crate) fn inv(v: Point) -> Point {
  [1. / v[0], 1. / v[1], 1. / v[2]]
}

pub(crate) fn add(lhs: Point, rhs: Point) -> Point {
  [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

pub(crate) fn add_s(v: Point, s: Size) -> Point {
  [v[0] + s, v[1] + s, v[2] + s]
}

pub(crate) fn sub(lhs: Point, rhs: Point) -> Point {
  [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

pub(crate) fn sub_s(v: Point, s: Size) -> Point {
  [v[0] - s, v[1] - s, v[2] - s]
}

pub(crate) fn mul(lhs: Point, rhs: Point) -> Point {
  [lhs[0] * rhs[0], lhs[1] * rhs[1], lhs[2] * rhs[2]]
}