#![allow(unused)]

use std::os::windows::prelude::MetadataExt;

use crate::{
    chunk_manager::{ChunkId, WorldPosition},
    ChunkData, CHUNK_SIZE,
};
use gamedata::material::Material;
use nalgebra_glm as glm;

use graphics::{Mesh, Vertex};

const CS: usize = CHUNK_SIZE;

pub fn generate_greedy_mesh(id: &ChunkId, data: &ChunkData) -> Mesh {
    let mut mesh = Mesh::default();
    let mut vertex_count = 0;

    // on each axis
    // from (x,y,z) to (d,u,v)
    for d0 in 0..3 {
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

                    let i0 = vertex_count as u32;

                    mesh.indices.extend(if orientation {
                        [i0 + 0, i0 + 3, i0 + 1, i0 + 0, i0 + 2, i0 + 3]
                    } else {
                        [i0 + 0, i0 + 1, i0 + 3, i0 + 0, i0 + 3, i0 + 2]
                    });

                    let ss = start;
                    let mut se = start;
                    se[u] = end[u];
                    let mut es = start;
                    es[v] = end[v];
                    let ee = end;

                    let c: glm::Vec3 = m.color().into();

                    mesh.vertices.extend([
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
                    vertex_count += 4;

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

    mesh
}

#[cfg(test)]
mod test {
    use test::Bencher;

    use crate::{chunk_generator::ChunkGenerator, chunk_manager::ChunkId};

    use super::generate_greedy_mesh;

    #[bench]
    fn single_chunk_meshing17(b: &mut Bencher) {
        let id = ChunkId::new(17, 17, 17);
        let data = ChunkGenerator::new().generate(&id);

        b.iter(|| {
            test::black_box(generate_greedy_mesh(&id, &data));
        });
    }

    #[bench]
    fn single_chunk_meshing0(b: &mut Bencher) {
        let id = ChunkId::new(0, 0, 0);
        let data = ChunkGenerator::new().generate(&id);

        b.iter(|| {
            test::black_box(generate_greedy_mesh(&id, &data));
        });
    }
}
