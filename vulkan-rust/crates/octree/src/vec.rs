#[allow(dead_code)]
use crate::octree::{Point, Size};

pub fn mul_s(v: Point, s: Size) -> Point {
    [v[0] * s, v[1] * s, v[2] * s]
}

pub fn mul_s_i(v: [i32; 3], s: i32) -> [i32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

pub fn div_s(v: Point, s: Size) -> Point {
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

pub fn to_f64(vec: Point) -> [f64; 3] {
    [vec[0] as f64, vec[1] as f64, vec[2] as f64]
}

pub fn i32_to_f32(vec: [i32; 3]) -> [f32; 3] {
    [vec[0] as f32, vec[1] as f32, vec[2] as f32]
}
