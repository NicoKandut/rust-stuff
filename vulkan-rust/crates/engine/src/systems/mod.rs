pub mod health;
pub mod physics;

use std::time::Duration;

use self::health::EntityHealth;
use self::physics::EntityPhysics;

pub trait System<T> {
    fn add(&mut self, entity: T);
    fn remove(&mut self, id: u64);
    fn act(&mut self, delta_time: f64) -> bool;
}

#[derive(Default, Debug)]
pub struct Entity {
    pub id: u64,
    pub movement: EntityPhysics,
    pub health: EntityHealth,
}

pub trait TimeBasedSystem<T> {
    fn add_entity(&mut self, entity: T);
    fn update(&mut self, delta_time: &Duration);
}