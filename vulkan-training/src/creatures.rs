use crate::math;
use crate::position::Moveable;
use crate::position::Pos3;

const GRAVITY: f64 = -9.81;
const TERMINAL_VELOCITY: f64 = -200.0;
const JUMP_DURATION: f64 = 1.0;

pub struct Creature {
  pub name: String,
  pub health: Health,
  pub position: Pos3,
  pub mass: f64,
  pub velocity: Pos3,
  pub acceleration: f64,
  pub grounded: bool,
  pub air_time: f64,
  pub jump_strength: f64,
}

pub struct Health {
  pub hp: f64,
  pub max_hp: f64,
}

impl Health {
  pub fn take_damage(&mut self, amount: f64) {
    self.hp -= math::min(amount, self.hp)
  }
  pub fn heal(&mut self, amount: f64) {
    self.hp += math::min(amount, self.max_hp - self.hp)
  }
}

pub trait CanJump {
  fn jump(&mut self, elapsed: f64);
}

impl Moveable for Creature {
  fn translate_by(&mut self, x: f64, y: f64, z: f64) {
    self.position.0 += x;
    self.position.1 += y;
    self.position.2 += z;
  }
}
