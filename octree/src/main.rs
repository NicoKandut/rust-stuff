use crate::generation::MaterialGenerator;

mod generation;
mod octree;
mod raycast;

fn main() {
    let gen = MaterialGenerator::new(17);

    let chunk = gen.get_chunk([0, 0, 0]);

    println!("Hello, world! \n {}", chunk[0]);
}
