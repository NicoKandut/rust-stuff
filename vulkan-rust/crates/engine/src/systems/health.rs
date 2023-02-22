use std::time::Duration;

use super::{System, TimeBasedSystem};

#[derive(Default, Debug)]
pub struct EntityHealth {
    pub id: u64,
    pub max: f32,
    pub current: f32,
    pub regeneration: f32,
}

#[derive(Default)]
pub struct HealthSystem {
    pub entities: Vec<EntityHealth>,
}

impl System<EntityHealth> for HealthSystem {
    fn add(&mut self, entity: EntityHealth) {
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

    fn act(&mut self, delta_time: f32) -> bool {
        let mut result = false;

        for entity in &mut self.entities {
            if entity.current < entity.max {
                entity.current = entity
                    .max
                    .min(entity.current + entity.regeneration * delta_time);

                result = true;
            }
        }

        result
    }
}

impl TimeBasedSystem<EntityHealth> for HealthSystem {
    fn add_entity(&mut self, entity: EntityHealth) {
        self.entities.push(entity)
    }

    fn update(&mut self, delta_time: &Duration) {
        for entity in &mut self.entities {
            if entity.current < entity.max {
                entity.current = entity
                    .max
                    .min(entity.current + entity.regeneration * delta_time.as_secs_f32());
            }
        }
    }
}
