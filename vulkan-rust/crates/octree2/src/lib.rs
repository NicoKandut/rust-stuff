#![feature(test)]

extern crate test;

use std::collections::HashMap;

use manager::NodeManager;
use node::Node;
use position::{Position, CHUNK_SIZE};

const AIR: usize = 0x8000000000000000;
const STONE: usize = 0x8000000000000001;

mod manager;
mod node;
mod position;

#[derive(Default)]
struct World {
    manager: NodeManager,
    chunk_roots: HashMap<Position, usize>,
}

impl World {
    pub fn get_block(&mut self, position: &Position) -> usize {
        let chunk_position = position.rounded_to(CHUNK_SIZE);

        match self.chunk_roots.get(&chunk_position) {
            Some(node) => {
                let mut current_node_id = *node;
                let mut current_size = CHUNK_SIZE;

                while !is_material(current_node_id) {
                    if let Some(node) = self.manager.get(&current_node_id) {
                        let relative_position = position.relative_to(current_size);
                        let child_index = position.to_child_index(current_size);
                        let child_id = node.get_child_id(child_index);
                        current_node_id = child_id;
                        current_size /= 2;
                    } else {
                        break;
                    }
                }

                current_node_id
            }
            None => AIR,
        }

        // todo
    }

    pub fn add_block(&mut self, position: &Position, material: usize) -> bool {
        let chunk_position = position.rounded_to(CHUNK_SIZE);

        let chunk_root_id = match self.chunk_roots.get(&chunk_position) {
            Some(node_id) => *node_id,
            None => {
                let node_id = self.manager.add(Node::default());
                self.chunk_roots.insert(position.clone(), node_id);
                node_id
            }
        };

        let mut current_node_id = chunk_root_id;
        let mut current_size = CHUNK_SIZE;
        let mut child_index = 0;

        while current_size > 2 {
            let relative_position = position.relative_to(current_size);
            child_index = relative_position.to_child_index(current_size);
            if let Some(node) = self.manager.get(&current_node_id) {
                let child_id = node.get_child_id(child_index);
                current_node_id = child_id;
            } else {
                let child_id = self.manager.add(Node::default());
                self.manager
                    .set_child_of(&current_node_id, &child_index, &child_id);
                current_node_id = child_id;
            }
            current_size /= 2;
        }

        let node = self
            .manager
            .get(&current_node_id)
            .expect("code above should add it in any case");

        if node.get_child_id(child_index) == material {
            return false;
        }

        let old_node = node.clone();
        let mut new_node = node.clone();
        new_node.set_child(&child_index, &material);

        self.manager.remove(&old_node);
        self.manager.add(new_node);

        return true;

        // todo
    }
}

fn is_material(node_id: usize) -> bool {
    node_id >= AIR
}

#[cfg(test)]
mod tests {
    use crate::{position::Position, World, STONE};

    #[test]
    fn adding_blocks_works() {
        let mut world = World::default();

        world.add_block(&Position::new(0, 0, 0), STONE);
        world.add_block(&Position::new(0, 10, 0), STONE);
        world.add_block(&Position::new(0, -10, 0), STONE);
    }
}
