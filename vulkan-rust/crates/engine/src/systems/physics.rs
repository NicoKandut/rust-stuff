use std::time::Duration;
use gamedata::vector::Vec3;
use octree::octree::Octree;
use super::{System, TimeBasedSystem};

#[derive(Default, Debug)]
pub struct EntityPhysics {
    pub id: u64,
    pub acceleration: Vec3,
    pub velocity: Vec3,
    pub position: Vec3,
    pub affected_by_gravity: bool,
    pub rotation: Vec3,
    pub half_size: Vec3,
}

pub struct PhysicsSystem {
    pub entities: Vec<EntityPhysics>,
    pub gravity_acceleration: f64,
    pub octree: Octree,
}

impl PhysicsSystem {
    pub fn new_earth_like(size: f32) -> Self {
        PhysicsSystem {
            entities: Vec::new(),
            gravity_acceleration: -9.81,
            octree: Octree::new(size),
        }
    }
}

impl System<EntityPhysics> for PhysicsSystem {
    fn add(&mut self, entity: EntityPhysics) {
        self.entities.push(entity)
    }

    fn remove(&mut self, id: u64) {
        let index = self
            .entities
            .iter()
            .position(|e| e.id == id)
            .expect("PositionSystem does not have that entity");

        self.entities.remove(index);
    }

    fn act(&mut self, delta_time: f64) -> bool {
        let mut result = false;
        for entity in &mut self.entities {
            if entity.affected_by_gravity {
                entity.acceleration.y = self.gravity_acceleration;
            }

            let delta_velocity = &entity.acceleration * delta_time;
            entity.velocity += &delta_velocity;

            let step = &entity.velocity * delta_time;
            entity.position += &step;

            if self.octree.intersects_box(&entity.position, &entity.half_size) {
                entity.position -= &step;
                entity.affected_by_gravity = false;
            } else {
                result = true;
            }

            // println!("  - Movement: {:?}", entity.position);
        }

        result
    }
}

impl TimeBasedSystem<EntityPhysics> for PhysicsSystem {
    fn add_entity(&mut self, entity: EntityPhysics) {
        self.entities.push(entity)
    }

    fn update(&mut self, delta_time: &Duration) {
        for entity in &mut self.entities {
            if entity.affected_by_gravity {
                entity.acceleration.y = self.gravity_acceleration;
            }

            let delta_velocity = &entity.acceleration * delta_time.as_secs_f64();
            entity.velocity += &delta_velocity;

            let step = &entity.velocity * delta_time.as_secs_f64();
            entity.position += &step;

            if self.octree.intersects_box(&entity.position, &entity.half_size) {
                entity.position -= &step;
                entity.affected_by_gravity = false;
            }
        }
    }
}
