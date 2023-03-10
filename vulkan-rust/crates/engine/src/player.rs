use crate::systems::{health::EntityHealth, physics::EntityPhysics};

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
                acceleration: glm::Vec3::default(),
                velocity: glm::Vec3::default(),
                position: glm::Vec3::new(0.0, 10.0, 0.0),
                affected_by_gravity: true,
                rotation: glm::Vec3::default(),
                half_size: glm::Vec3::default(),
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
