use std::time::Duration;

use gamedata::{material::Material, vector::Vec3};
use octree::octree::Node;


use crate::{
    player::Player,
    systems::{health::HealthSystem, physics::PhysicsSystem, TimeBasedSystem},
};

pub struct GameState {
    next_id: u64,
    pub physics: PhysicsSystem,
    health: HealthSystem,
}

#[derive(Debug)]
pub struct Voxel {
    pub position: Vec3,
    pub material: Material,
}

impl Voxel {
    pub(crate) fn new(position: Vec3, material: Material) -> Voxel {
        Self { position, material }
    }
}

impl GameState {
    pub fn new(size: f32) -> Self {
        Self {
            next_id: 0,
            physics: PhysicsSystem::new_earth_like(size),
            health: HealthSystem::default(),
        }
    }

    pub fn get_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        id
    }

    pub fn update(&mut self, delta_time: &Duration) {
        self.physics.update(delta_time);
        self.health.update(delta_time);
    }

    pub fn add_player(&mut self, player: Player) {
        self.physics.add_entity(player.movement);
        self.health.add_entity(player.health);
    }

    pub(crate) fn get_world_data(&self) -> Vec<&Node> {
        self.physics.octree.get_leaves()
    }
}
