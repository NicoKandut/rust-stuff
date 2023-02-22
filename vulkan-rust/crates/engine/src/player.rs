use crate::systems::{health::EntityHealth, physics::EntityPhysics};
use gamedata::vector::Vec3;

pub struct Player {
    pub id: u64,
    pub movement: EntityPhysics,
    pub health: EntityHealth,
}

impl Player {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            movement: EntityPhysics {
                id,
                acceleration: Vec3::default(),
                velocity: Vec3::default(),
                position: Vec3::new(0.0, 10.0, 0.0),
                affected_by_gravity: true,
                rotation: Vec3::default(),
                half_size: Vec3::default(),
            },
            health: EntityHealth {
                id,
                max: 100.0,
                current: 50.0,
                regeneration: 10.0,
            },
        }
    }
}
