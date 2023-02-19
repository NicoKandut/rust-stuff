#![feature(test)]
#![feature(portable_simd)]

extern crate test;

use crate::octree::{Material, Octree};

mod octree;
mod benches;
mod vec;

fn main() {
    build_complete_tree(5).expect("Tree operations failed...");
}

fn build_complete_tree(depth: usize) -> Result<(), ()> {
    let mut tree = Octree::new(1.0);

    for level in 0..depth {
        let split_end = tree.len();
        let split_start = split_end - 8usize.pow(level as u32);

        for node_index in split_start..split_end {
            let child_index = tree.split_node(node_index)?;
            for i in 0..8 {
                let material = if i % 2 == 0 {
                    Material::Air
                } else {
                    Material::Stone
                };
                tree.set(child_index + i, material)?;
            }
        }
    }

    println!("Done");

    Ok(())
}
