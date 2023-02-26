#![allow(unused)]

use std::os::windows::prelude::MetadataExt;

use gamedata::material::Material;
use nalgebra_glm as glm;
use world::{
    chunk_manager::{ChunkId, WorldPosition},
    fixed_tree::ChunkData,
    CHUNK_SIZE,
};

use super::Vertex;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

const CS: usize = CHUNK_SIZE;

pub fn generate_greedy_mesh(id: &ChunkId, data: &ChunkData, first_vertex: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let world_pos = WorldPosition::from(id.clone());

    // on each axis
    // from (x,y,z) to (d,u,v)
    for d0 in 0..3 {
        // println!("Dir: {}", d0);
        let d = d0 % 3;
        let u = (d0 + 1) % 3;
        let v = (d0 + 2) % 3;

        let mut x = [0; 3];
        let mut x2 = [0; 3];
        x[d] = 0;

        // look at all layers
        while x[d] < (CS + 1) {
            x[u] = 0;
            x[v] = 0;
            // build the mask
            let mut mask = [None; CS * CS];
            let mut n = 0;
            while x[v] < CS {
                x[u] = 0;
                while x[u] < CS {
                    x2 = x;
                    x2[d] -= 1;
                    let cur_mat = data.get(x[0], x[1], x[2]);
                    let next_mat = data.get(x2[0], x2[1], x2[2]);

                    let face_type_c = match (cur_mat, next_mat) {
                        (None, None) => None,
                        (Some(m), None) => Some((m, true)),
                        (None, Some(m)) => Some((m, false)),
                        (Some(m1), Some(m2)) => match (m1.is_opaque(), m2.is_opaque()) {
                            (true, true) => None,
                            (true, false) => Some((m1, true)),
                            (false, true) => Some((m2, false)),
                            (false, false) => None,
                        },
                    };

                    mask[n] = face_type_c;
                    n += 1;
                    x[u] += 1;
                }
                x[v] += 1;
            }

            let mut start = [0_usize; 3];
            start[d] = x[d] as usize;
            let mut n = 0;

            for mask_v in 0..CS {
                start[v] = mask_v;
                start[u] = 0;
                for mask_u in 0..CS {
                    if mask[n] == None {
                        n += 1;
                        continue;
                    }

                    start[u] = mask_u;
                    let mut end = start;

                    // find length in dim1
                    while end[u] < CS
                        && mask[end[v] * CS + end[u]] == mask[start[v] * CS + start[u]]
                    {
                        end[u] += 1;
                    }
                    let u_end = end[u];

                    end = start;
                    end[v] += 1;

                    // find length in dim2
                    while end[v] < CS && mask[end[v] * CS + end[u]] == mask[start[v] + start[u]] {
                        while end[u] < u_end
                            && mask[end[v] * CS + end[u]] == mask[start[v] * CS + start[u]]
                        {
                            end[u] += 1;
                        }

                        if (end[u] < u_end) {
                            break;
                        }

                        end[u] = start[u];
                        end[v] += 1;
                    }
                    end[u] = u_end;

                    let (m, orientation) = mask[start[v] * CS + start[u]].unwrap();
                    let mut normal = [0, 0, 0];
                    normal[d] = 1 - 2 * (orientation as i32);
                    let normal = glm::vec3(normal[0] as f32, normal[1] as f32, normal[2] as f32);

                    let i0 = first_vertex + vertices.len() as u32;
                    indices.extend(if orientation {
                        [i0 + 0, i0 + 3, i0 + 1, i0 + 0, i0 + 2, i0 + 3]
                    } else {
                        [i0 + 0, i0 + 1, i0 + 3, i0 + 0, i0 + 3, i0 + 2]
                    });

                    let global_start = [
                        world_pos.x + start[0] as i32,
                        world_pos.y + start[1] as i32,
                        world_pos.z + start[2] as i32,
                    ];
                    let global_end = [
                        world_pos.x + end[0] as i32,
                        world_pos.y + end[1] as i32,
                        world_pos.z + end[2] as i32,
                    ];

                    let ss = global_start;
                    let mut se = global_start;
                    se[u] = global_end[u];
                    let mut es = global_start;
                    es[v] = global_end[v];
                    let ee = global_end;

                    let c: glm::Vec3 = m.color().into();

                    vertices.extend([
                        Vertex::new(
                            glm::vec3(ss[0] as f32, ss[1] as f32, ss[2] as f32),
                            c,
                            normal,
                        ),
                        Vertex::new(
                            glm::vec3(se[0] as f32, se[1] as f32, se[2] as f32),
                            c,
                            normal,
                        ),
                        Vertex::new(
                            glm::vec3(es[0] as f32, es[1] as f32, es[2] as f32),
                            c,
                            normal,
                        ),
                        Vertex::new(
                            glm::vec3(ee[0] as f32, ee[1] as f32, ee[2] as f32),
                            c,
                            normal,
                        ),
                    ]);

                    for vm in start[v]..end[v] {
                        for um in start[u]..end[u] {
                            mask[vm * CS + um] = None;
                        }
                    }

                    n += 1;
                }
            }

            x[d] += 1;
        }
    }

    // println!("Meshing: vOffset: {first_vertex}, vLen: {}", vertices.len());

    Mesh { vertices, indices }
}

#[cfg(test)]
mod test {
    use test::Bencher;

    use world::{chunk_generator::ChunkGenerator, chunk_manager::ChunkId};

    use super::generate_greedy_mesh;

    #[bench]
    fn single_chunk_meshing17(b: &mut Bencher) {
        let id = ChunkId::new(17, 17, 17);
        let data = ChunkGenerator::new().generate(&id);
        let first_vertex = 16;

        b.iter(|| {
            test::black_box(generate_greedy_mesh(&id, &data, first_vertex));
        });
    }

    #[bench]
    fn single_chunk_meshing0(b: &mut Bencher) {
        let id = ChunkId::new(0, 0, 0);
        let data = ChunkGenerator::new().generate(&id);
        let first_vertex = 0;

        b.iter(|| {
            test::black_box(generate_greedy_mesh(&id, &data, first_vertex));
        });
    }
}
