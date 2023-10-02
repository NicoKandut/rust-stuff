#![allow(unused)]

use std::os::windows::prelude::MetadataExt;

use nalgebra_glm as glm;

use gamedata::material::Material;
use graphics::{Mesh, Vertex};

use crate::{
    chunk_id::ChunkId, slice::CubeSlice, traits::Data3D, ChunkData, CHUNK_SIZE, CHUNK_SIZE_SAFE,
    CHUNK_SIZE_SQUARED,
};

pub fn generate_greedy_mesh<T>(id: &ChunkId, data: &T) -> Mesh
where
    T: Data3D<Material>,
{
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
        x[d] = 2;

        // look at all layers
        while x[d] < CHUNK_SIZE_SAFE {
            x[u] = 1;
            x[v] = 1;
            // build the mask
            let mut mask = [None; CHUNK_SIZE_SQUARED];
            let mut n = 0;
            while x[v] < (CHUNK_SIZE_SAFE - 1) {
                x[u] = 1;
                while x[u] < (CHUNK_SIZE_SAFE - 1) {
                    x2 = x;
                    x2[d] -= 1;
                    let cur_mat = data.get(x[0], x[1], x[2]);
                    let prev_mat = data.get(x2[0], x2[1], x2[2]);
                    let face_type_c = match (cur_mat.is_opaque(), prev_mat.is_opaque()) {
                        (true, true) => None,
                        (true, false) => Some((cur_mat, true)),
                        (false, true) => Some((prev_mat, false)),
                        (false, false) => None,
                    };

                    mask[n] = face_type_c;
                    n += 1;
                    x[u] += 1;
                }
                x[v] += 1;
            }

            let mut start = [0_usize; 3];
            start[d] = x[d] - 1 as usize;
            let mut n = 0;

            for mask_v in 0..CHUNK_SIZE {
                start[v] = mask_v;
                start[u] = 0;
                for mask_u in 0..CHUNK_SIZE {
                    if mask[n] == None {
                        n += 1;
                        continue;
                    }

                    start[u] = mask_u;
                    let mut end = start;

                    // find length in dim1
                    while end[u] < CHUNK_SIZE
                        && mask[end[v] * CHUNK_SIZE + end[u]]
                            == mask[start[v] * CHUNK_SIZE + start[u]]
                    {
                        end[u] += 1;
                    }
                    let u_end = end[u];

                    end = start;
                    end[v] += 1;

                    // find length in dim2
                    while end[v] < CHUNK_SIZE
                        && mask[end[v] * CHUNK_SIZE + end[u]] == mask[start[v] + start[u]]
                    {
                        while end[u] < u_end
                            && mask[end[v] * CHUNK_SIZE + end[u]]
                                == mask[start[v] * CHUNK_SIZE + start[u]]
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

                    let (m, orientation) = mask[start[v] * CHUNK_SIZE + start[u]].unwrap();
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

                    let vertices = [
                        [ss[0] as f32, ss[1] as f32, ss[2] as f32],
                        [se[0] as f32, se[1] as f32, se[2] as f32],
                        [es[0] as f32, es[1] as f32, es[2] as f32],
                        [ee[0] as f32, ee[1] as f32, ee[2] as f32],
                    ]
                    .map(|position| glm::vec3(position[0], position[1], position[2]))
                    .map(|position| Vertex::from_material(position, m.into(), normal));

                    mesh.vertices.extend(vertices);
                    vertex_count += 4;

                    for vm in start[v]..end[v] {
                        for um in start[u]..end[u] {
                            mask[vm * CHUNK_SIZE + um] = None;
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

/// TODO: cleanup
pub fn generate_greedy_mesh_water<T>(id: &ChunkId, data: &T) -> Mesh
where
    T: Data3D<Material>,
{
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
        x[d] = 2;

        // look at all layers
        while x[d] < CHUNK_SIZE_SAFE {
            x[u] = 1;
            x[v] = 1;
            // build the mask
            let mut mask = [None; CHUNK_SIZE_SQUARED];
            let mut n = 0;
            while x[v] < (CHUNK_SIZE_SAFE - 1) {
                x[u] = 1;
                while x[u] < (CHUNK_SIZE_SAFE - 1) {
                    x2 = x;
                    x2[d] -= 1;
                    let cur_mat = data.get(x[0], x[1], x[2]);
                    let prev_mat = data.get(x2[0], x2[1], x2[2]);
                    let face_type_c = if cur_mat == Material::Water && prev_mat.is_invisible() {
                        Some((cur_mat, true))
                    } else if cur_mat.is_invisible() && prev_mat == Material::Water {
                        Some((prev_mat, false))
                    } else {
                        None
                    };

                    mask[n] = face_type_c;
                    n += 1;
                    x[u] += 1;
                }
                x[v] += 1;
            }

            let mut start = [0_usize; 3];
            start[d] = x[d] - 1 as usize;
            let mut n = 0;

            for mask_v in 0..CHUNK_SIZE {
                start[v] = mask_v;
                start[u] = 0;
                for mask_u in 0..CHUNK_SIZE {
                    if mask[n] == None {
                        n += 1;
                        continue;
                    }

                    start[u] = mask_u;
                    let mut end = start;

                    // find length in dim1
                    while end[u] < CHUNK_SIZE
                        && mask[end[v] * CHUNK_SIZE + end[u]]
                            == mask[start[v] * CHUNK_SIZE + start[u]]
                    {
                        end[u] += 1;
                    }
                    let u_end = end[u];

                    end = start;
                    end[v] += 1;

                    // find length in dim2
                    while end[v] < CHUNK_SIZE
                        && mask[end[v] * CHUNK_SIZE + end[u]] == mask[start[v] + start[u]]
                    {
                        while end[u] < u_end
                            && mask[end[v] * CHUNK_SIZE + end[u]]
                                == mask[start[v] * CHUNK_SIZE + start[u]]
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

                    let (m, orientation) = mask[start[v] * CHUNK_SIZE + start[u]].unwrap();
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

                    let vertices = [
                        [ss[0] as f32, ss[1] as f32, ss[2] as f32],
                        [se[0] as f32, se[1] as f32, se[2] as f32],
                        [es[0] as f32, es[1] as f32, es[2] as f32],
                        [ee[0] as f32, ee[1] as f32, ee[2] as f32],
                    ]
                    .map(|position| glm::vec3(position[0], position[1], position[2]))
                    .map(|position| Vertex::from_material(position, m.into(), normal));

                    mesh.vertices.extend(vertices);
                    vertex_count += 4;

                    for vm in start[v]..end[v] {
                        for um in start[u]..end[u] {
                            mask[vm * CHUNK_SIZE + um] = None;
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

    use gamedata::material::Material;

    use crate::{
        chunk_id::ChunkId,
        gen::chunk::Chunk,
        seed::{PositionalSeed, WorldSeed},
        slice::CubeSlice,
        traits::{Data3D, Generate, Voxelize},
        ChunkSeed, CHUNK_SIZE_SAFE,
    };

    use super::generate_greedy_mesh;

    const WORLD_SEED: WorldSeed = WorldSeed::new(17);

    #[test]
    fn empty() {
        let id = ChunkId::new(17, 17, 17);
        let mut data = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();
        let mesh = generate_greedy_mesh(&id, &data);
        assert_eq!(mesh.vertices.len(), 0);
        assert_eq!(mesh.indices.len(), 0);
    }

    #[test]
    fn borders_mesh_to_empty() {
        let id = ChunkId::new(17, 17, 17);
        let mut data = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();
        for d in 0..3 {
            let u = (d + 1) % 3;
            let v = (d + 2) % 3;
            let mut x = [0, 0, 0];
            for i in 0..65 {
                x[d] = i;

                x[u] = 0;
                x[v] = 0;
                data.set(x[0], x[1], x[2], Material::Stone);
                x[u] = 0;
                x[v] = 65;
                data.set(x[0], x[1], x[2], Material::Stone);
                x[u] = 65;
                x[v] = 0;
                data.set(x[0], x[1], x[2], Material::Stone);
                x[u] = 65;
                x[v] = 65;
                data.set(x[0], x[1], x[2], Material::Stone);
            }
        }

        let mesh = generate_greedy_mesh(&id, &data);
        assert_eq!(mesh.vertices.len(), 0);
        assert_eq!(mesh.indices.len(), 0);
    }

    #[test]
    fn single_block() {
        let id = ChunkId::new(17, 17, 17);
        let mut data = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();
        data.set(1, 1, 1, Material::Stone);
        let mesh = generate_greedy_mesh(&id, &data);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        for v in mesh.vertices {
            assert!(v.pos_mat.x >= 0.0 && v.pos_mat.x <= 1.0);
            assert!(v.pos_mat.y >= 0.0 && v.pos_mat.y <= 1.0);
            assert!(v.pos_mat.z >= 0.0 && v.pos_mat.z <= 1.0);
            assert!(v.pos_mat.w == f32::from(u8::from(Material::Stone)));
        }
    }

    #[test]
    fn greedy_block_2() {
        let id = ChunkId::new(17, 17, 17);
        let mut data = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();
        data.set(1, 1, 1, Material::Stone);
        data.set(1, 1, 2, Material::Stone);
        data.set(1, 2, 1, Material::Stone);
        data.set(1, 2, 2, Material::Stone);
        data.set(2, 1, 1, Material::Stone);
        data.set(2, 1, 2, Material::Stone);
        data.set(2, 2, 1, Material::Stone);
        data.set(2, 2, 2, Material::Stone);
        let mesh = generate_greedy_mesh(&id, &data);
        assert_eq!(mesh.vertices.len(), 24);
        assert_eq!(mesh.indices.len(), 36);
        for v in mesh.vertices {
            assert!(v.pos_mat.x >= 0.0 && v.pos_mat.x <= 2.0);
            assert!(v.pos_mat.y >= 0.0 && v.pos_mat.y <= 2.0);
            assert!(v.pos_mat.z >= 0.0 && v.pos_mat.z <= 2.0);
            assert!(v.pos_mat.w == f32::from(u8::from(Material::Stone)));
        }
    }

    #[bench]
    fn single_chunk_meshing17(b: &mut Bencher) {
        let id = ChunkId::new(17, 17, 17);
        let chunk_seed = ChunkSeed::new(&WORLD_SEED, &id);
        let data = Chunk::generate(chunk_seed).voxelize();

        b.iter(|| {
            test::black_box(generate_greedy_mesh(&id, &data.voxels));
        });
    }

    #[bench]
    fn single_chunk_meshing0(b: &mut Bencher) {
        let id = ChunkId::new(0, 0, 0);
        let chunk_seed = ChunkSeed::new(&WORLD_SEED, &id);
        let data = Chunk::generate(chunk_seed).voxelize();

        b.iter(|| {
            test::black_box(generate_greedy_mesh(&id, &data.voxels));
        });
    }
}
