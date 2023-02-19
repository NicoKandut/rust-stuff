use crate::position::Pos3;

pub struct Physics {
  pub mass: f64,
  pub velocity: Pos3,
  pub acceleration: f64,
  pub grounded: bool,
  pub air_time: f64,
}

pub trait WithGravity {
  fn fall(&mut self, elapsed: f64);
}

pub trait WithForce {
  fn apply_force(&mut self, amount: Pos3);
}
