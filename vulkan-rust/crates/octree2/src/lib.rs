#![feature(test)]

extern crate test;

use std::collections::HashMap;

use manager::NodeManager;
use node::Node;
use position::{Position, CHUNK_SIZE};

pub const AIR: usize = 0x8000000000000000;
pub const STONE: usize = 0x8000000000000001;
pub const CHANGING: usize = 0x8000000000000002;

mod manager;
mod node;
mod position;

#[derive(Default)]
pub struct World {
    manager: NodeManager,
    chunk_roots: HashMap<Position, usize>,
}

impl World {
    pub fn get_block(&mut self, position: &Position) -> usize {
        println!("Getting at {position:?}");
        let chunk_position = position.rounded_to(CHUNK_SIZE);

        match self.chunk_roots.get(&chunk_position) {
            Some(chunk_id) => {
                let mut current_node_id = *chunk_id;
                let mut current_size = CHUNK_SIZE;

                while !is_material(current_node_id) && current_size > 1 {
                    if let Some(node) = self.manager.get(&current_node_id) {
                        let relative_position = position.relative_to(current_size);
                        let child_index = relative_position.to_child_index(current_size);
                        let child_id = node.get_child_id(child_index);
                        current_node_id = child_id;
                        current_size /= 2;
                    } else {
                        println!("Tried to access non-exiting node ");
                        break;
                    }
                }

                println!("  FINAL: Size: {current_size}, NodeId: {current_node_id}");

                current_node_id
            }
            None => AIR,
        }

        // todo
    }

    pub fn add_block(&mut self, position: &Position, material: usize) -> bool {
        let chunk_position = position.rounded_to(CHUNK_SIZE);
        println!("Chunk position: {chunk_position:?}");
        let chunk_root_id = self.get_chunk_root(chunk_position);
        println!("Found chunk {chunk_root_id}");

        let mut current_node_id = chunk_root_id;
        let mut current_size = CHUNK_SIZE;
        let mut child_index = 0;

        while current_size > 2 {
            let relative_position = position.relative_to(current_size);
            child_index = relative_position.to_child_index(current_size);
            println!("Lookup: relative: {:?}", relative_position);
            current_node_id = match self.manager.get(&current_node_id) {
                Some(node) => {
                    let child_id = node.get_child_id(child_index);
                    if is_material(child_id) {
                        self.manager
                            .set_child_of(&current_node_id, &child_index, &CHANGING);
                        let mut new_node = Node::new_air();
                        new_node.set_child(&child_index, &child_id);
                        println!("  Found material, expanding deeper {new_node:?}");
                        let new_child_id = self.manager.add(new_node);
                        self.manager
                            .set_child_of(&current_node_id, &child_index, &new_child_id);

                        new_child_id
                    } else {
                        println!("  Node exists, lookup success");
                        child_id
                    }
                }
                None => {
                    println!("  Failed, adding new node as child of {current_node_id}");
                    let child_id = self.manager.add(Node::new_air());
                    self.manager
                        .set_child_of(&current_node_id, &child_index, &child_id);
                    child_id
                }
            };
            current_size /= 2;
        }

        let node = self
            .manager
            .get(&current_node_id)
            .expect("code above should add it in any case");

        let relative_position = position.relative_to(current_size);
        child_index = relative_position.to_child_index(current_size);
        if node.get_child_id(child_index) == material {
            println!("Replacing here");
        }

        println!("Inserting {material} at {position:?}");

        self.manager
            .set_child_of(&current_node_id, &child_index, &material);

        println!("CHUNKS: {:#?}", self.chunk_roots);
        println!("NODES: {:#?}", self.manager.nodes());

        return true;

        // todo
    }

    fn get_chunk_root(&mut self, chunk_position: Position) -> usize {
        match self.chunk_roots.get(&chunk_position) {
            Some(node_id) => *node_id,
            None => self.add_chunk_root(&chunk_position),
        }
    }

    fn add_chunk_root(&mut self, position: &Position) -> usize {
        let node_id = self.manager.add(Node::new_air());
        self.chunk_roots.insert(position.clone(), node_id);

        node_id
    }
}

fn is_material(node_id: usize) -> bool {
    node_id >= AIR
}

#[cfg(test)]
mod tests {
    use crate::{position::Position, World, AIR, STONE};

    #[test]
    fn adding_blocks_works() {
        let mut world = World::default();

        let p1 = Position::new(0, 0, 0);
        let p2 = Position::new(0, 10, 0);
        let p3 = Position::new(0, 0, -10);
        world.add_block(&p1, STONE);
        world.add_block(&p2, STONE);
        world.add_block(&p3, STONE);
        assert_eq!(world.get_block(&p1), STONE, "Assertion failed at {:?}", p1);
        assert_eq!(world.get_block(&p2), STONE, "Assertion failed at {:?}", p2);
        assert_eq!(world.get_block(&p3), STONE, "Assertion failed at {:?}", p3);

        for z in -10..10 {
            for y in -10..10 {
                for x in -10..10 {
                    let p = Position::new(x, y, z);
                    let material = world.get_block(&p);
                    if p == p1 || p == p2 || p == p3 {
                        assert_eq!(material, STONE, "Expected STONE at {:?}", p);
                    } else {
                        assert_eq!(material, AIR, "Expected AIR at {:?}", p);
                    };
                }
            }
        }
    }
}
