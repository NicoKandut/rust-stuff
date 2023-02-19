use crate::position::Pos3;
use crate::Health;

const GRAVITY: f64 = -9.81;
const TERMINAL_VELOCITY: f64 = -200.0;
const JUMP_DURATION: f64 = 1.0;

pub trait Gravity {}

pub struct SystemBase<E> {
  pub entities: Vec<E>,
}

pub trait System {
  fn act(&self, elapsed: f64);
}

pub struct GravityEntity {
  position: Pos3,
  velocity: Pos3,
  grounded: bool,
  air_time: f64,
}

pub type GravitySystem = SystemBase<GravityEntity>;

impl System for GravitySystem {
  fn act(&self, elapsed: f64) {
    for entity in self.entities.iter_mut() {
      if entity.grounded {
        return;
      }

      entity.air_time += elapsed;
      entity.velocity.2 += GRAVITY * elapsed;

      if entity.velocity.2 < TERMINAL_VELOCITY {
        entity.velocity.2 = TERMINAL_VELOCITY;
      }

      entity.position.2 += entity.velocity.2 * elapsed;

      if entity.position.2 < 0.0 {
        entity.position.2 = 0.0;
        entity.air_time = 0.0;
        entity.grounded = true;
      }
    }
  }
}

pub struct RegenerationEntity {
  health: Health,
}

pub type RegenerationSystem = SystemBase<RegenerationEntity>;

impl System for RegenerationSystem {
  fn act(&self, elapsed: f64) {
    for entity in self.entities.iter_mut() {
      entity.health.heal(1.0 * elapsed);
    }
  }
}
