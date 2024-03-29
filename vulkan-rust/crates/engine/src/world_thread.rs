use crate::chunk_stream::{ChunkAction, ChunkTracker};
use gamedata::material::Material;
use geometry::Ray;
use graphics::Mesh;
use logging::{log, LOG_WORLD};
use rayon::prelude::*;
use std::{
    collections::HashSet,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use threadpool::ThreadPool;
use world::{
    gen::chunk::{compress, Chunk},
    mesh_generator::{generate_greedy_mesh, generate_greedy_mesh_water},
    slice::CubeSlice,
    traits::{Data3D, Generate, Voxelize},
    ChunkId, ChunkSeed, Raycast, World, WorldPosition, WorldSeed, CHUNK_SIZE, CHUNK_SIZE_I,
    CHUNK_SIZE_SAFE,
};

const REMESH_INTERVAL: Duration = Duration::from_millis(100);

pub(crate) enum ModifyAction {
    Remove,
    Place(Material),
}

pub(crate) enum Request {
    Move(glm::Vec3),
    SetRenderDistance(f32, f32),
    Modify {
        ray: Ray,
        range: f32,
        action: ModifyAction,
    },
    Exit,
}

pub(crate) enum MeshEvent {
    Add(ChunkId, (Option<Mesh>, Option<Mesh>)),
    Remove(ChunkId),
}

pub(crate) fn spawn() -> (
    thread::JoinHandle<()>,
    mpsc::Sender<Request>,
    mpsc::Receiver<MeshEvent>,
) {
    let (in_tx, in_rx) = mpsc::channel();
    let (out_tx, out_rx) = mpsc::channel();
    let handle = thread::Builder::new()
        .name("world_thread".to_owned())
        .spawn(move || {
            log!(*LOG_WORLD, "World Thread started");

            let seed = WorldSeed::new(17);
            let mut world = World::new(seed.clone());
            let mut dirty_chunks = HashSet::default();
            let mut overflow = Vec::new();
            let mut chunk_stream = ChunkTracker::new(0.0, 0.0);
            let thread_pool = ThreadPool::new("meshing_thread", 8);
            let mut next_remesh = Instant::now();

            'thread: loop {
                'recv: while let Ok(request) = in_rx.recv_deadline(next_remesh) {
                    match request {
                        Request::Move(center) => {
                            let actions = chunk_stream.set_center(center);
                            log!(
                                *LOG_WORLD,
                                "Player moved, {} chunk actions needed",
                                actions.len()
                            );
                            process_chunk_actions(
                                actions,
                                &mut world,
                                &out_tx,
                                &thread_pool,
                                &mut dirty_chunks,
                                &mut overflow,
                            );
                        }
                        Request::SetRenderDistance(load_distance, unload_distance) => {
                            let actions =
                                chunk_stream.set_distances(load_distance, unload_distance);
                            log!(
                                *LOG_WORLD,
                                "Render distance updated, {} chunk actions needed",
                                actions.len()
                            );
                            process_chunk_actions(
                                actions,
                                &mut world,
                                &out_tx,
                                &thread_pool,
                                &mut dirty_chunks,
                                &mut overflow,
                            );
                        }
                        Request::Modify { ray, range, action } => {
                            log!(*LOG_WORLD, "Player attempts modification");
                            if let Some(distance) = world.cast_ray(&ray, &(0.0..range)) {
                                let (correction, material) = match action {
                                    ModifyAction::Remove => (0.01, Material::Unset),
                                    ModifyAction::Place(material) => (-0.01, material),
                                };
                                let hit = ray.point_on_ray(distance + correction);
                                let position = WorldPosition::from(&hit);
                                let chunk_id = ChunkId::from(&position);
                                let data = world
                                    .chunk_manager
                                    .get_mut(&chunk_id)
                                    .expect("Should be impossible to modify unloaded chunk");
                                let position_in_chunk = position.rem_euclid(CHUNK_SIZE_I);
                                data.set(
                                    position_in_chunk.x as usize,
                                    position_in_chunk.y as usize,
                                    position_in_chunk.z as usize,
                                    material,
                                );

                                let adjecent = chunk_id.get_adjecent();
                                dirty_chunks.insert(chunk_id);

                                if position_in_chunk.x == 0 {
                                    if world.chunk_manager.get(&adjecent[0]).is_some() {
                                        dirty_chunks.insert(adjecent[0]);
                                    }
                                } else if position_in_chunk.x == CHUNK_SIZE_I - 1 {
                                    if world.chunk_manager.get(&adjecent[1]).is_some() {
                                        dirty_chunks.insert(adjecent[1]);
                                    }
                                }

                                if position_in_chunk.y == 0 {
                                    if world.chunk_manager.get(&adjecent[2]).is_some() {
                                        dirty_chunks.insert(adjecent[2]);
                                    }
                                } else if position_in_chunk.y == CHUNK_SIZE_I - 1 {
                                    if world.chunk_manager.get(&adjecent[3]).is_some() {
                                        dirty_chunks.insert(adjecent[3]);
                                    }
                                }

                                if position_in_chunk.z == 0 {
                                    if world.chunk_manager.get(&adjecent[4]).is_some() {
                                        dirty_chunks.insert(adjecent[4]);
                                    }
                                } else if position_in_chunk.z == CHUNK_SIZE_I - 1 {
                                    if world.chunk_manager.get(&adjecent[5]).is_some() {
                                        dirty_chunks.insert(adjecent[5]);
                                    }
                                }
                            }

                            break 'recv;
                        }
                        Request::Exit => break 'thread,
                    }
                }

                mesh_dirty_chunks(&mut dirty_chunks, &world, &out_tx);
                next_remesh += REMESH_INTERVAL;
            }

            log!(*LOG_WORLD, "World Thread exited");
        })
        .expect("World Thread is mandatory");

    (handle, in_tx, out_rx)
}

fn mesh_dirty_chunks(
    dirty_chunks: &mut HashSet<ChunkId>,
    world: &World,
    out_tx: &mpsc::Sender<MeshEvent>,
) {
    if !dirty_chunks.is_empty() {
        log!(*LOG_WORLD, "{} dirty chunks", dirty_chunks.len());
    }

    dirty_chunks
        .par_drain()
        .filter_map(|id| match world.chunk_manager.get(&id) {
            Some(_) => Some(MeshEvent::Add(id, remesh(&id, &world))),
            None => Some(MeshEvent::Remove(id)),
        })
        .collect::<Vec<_>>()
        .into_iter()
        .for_each(|event| {
            out_tx.send(event).expect("Render Thread must be available");
        })
}

fn process_chunk_actions(
    actions: Vec<ChunkAction>,
    world: &mut World,
    out_tx: &mpsc::Sender<MeshEvent>,
    thread_pool: &ThreadPool,
    dirty_chunks: &mut HashSet<ChunkId>,
    overflow_set: &mut Vec<(i32, i32, i32, Material)>,
) {
    let outcomes = actions
        .into_par_iter()
        .map(|action| match action {
            ChunkAction::Load(id) => {
                let seed = ChunkSeed::new(&world.seed, &id);
                let id = *seed.id();
                let chunk = Chunk::generate(seed);
                let generated_chunk = chunk.voxelize();
                let chunk_data = compress(&generated_chunk.voxels);
                (id, Some(chunk_data), Some(generated_chunk.overflow))
            }
            ChunkAction::Unload(id) => (id, None, None),
        })
        .collect::<Vec<_>>();

    outcomes.into_iter().for_each(|(id, data, overflow)| {
        dirty_chunks.insert(id);

        match data {
            Some(data) => {
                world.chunk_manager.insert(&id, data);
                [
                    ChunkId::new(id.x - 1, id.y, id.z),
                    ChunkId::new(id.x, id.y - 1, id.z),
                    ChunkId::new(id.x, id.y, id.z - 1),
                ]
                .into_iter()
                .filter(|id| world.chunk_manager.get(id).is_some())
                .for_each(|id| {
                    dirty_chunks.insert(id);
                });
            }
            None => world.chunk_manager.remove(&id),
        }

        if let Some(overflow) = overflow {
            overflow_set.extend(overflow);
        }
    });

    overflow_set.retain(
        |(x, y, z, m)| match world.chunk_manager.set_block(*x, *y, *z, *m) {
            Ok(id) => {
                dirty_chunks.insert(id);
                false
            }
            Err(_) => true,
        },
    )
}

fn remesh(id: &ChunkId, world: &World) -> (Option<Mesh>, Option<Mesh>) {
    if let Some(chunk_data) = world.chunk_manager.get(id) {
        // read old data
        let mut blocks = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();
        let mut contains_opaque_blocks = false;
        let mut contains_invisible_blocks = false;

        // center chunk
        for x in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let material = chunk_data.get(x, y, z);
                    blocks.set(x + 1, y + 1, z + 1, material);
                    contains_opaque_blocks |= material.is_opaque();
                    contains_invisible_blocks |= !material.is_opaque();
                }
            }
        }

        let adjecent = id.get_adjecent();

        // x
        // if let Some(adjecent) = world.chunk_manager.get(&adjecent[0]) {
        //     for y in 0..CHUNK_SIZE {
        //         for z in 0..CHUNK_SIZE {
        //             let material = adjecent.get(CHUNK_SIZE - 1, y, z);
        //             blocks.set(0, y + 1, z + 1, material);
        //             contains_opaque_blocks |= material.is_opaque();
        //             contains_invisible_blocks |= !material.is_opaque();
        //         }
        //     }
        // }
        if let Some(adjecent) = world.chunk_manager.get(&adjecent[1]) {
            for y in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let material = adjecent.get(0, y, z);
                    blocks.set(CHUNK_SIZE_SAFE - 1, y + 1, z + 1, material);
                    contains_opaque_blocks |= material.is_opaque();
                    contains_invisible_blocks |= !material.is_opaque();
                }
            }
        }

        // y
        // if let Some(adjecent) = world.chunk_manager.get(&adjecent[2]) {
        //     for x in 0..CHUNK_SIZE {
        //         for z in 0..CHUNK_SIZE {
        //             let material = adjecent.get(x, CHUNK_SIZE - 1, z);
        //             blocks.set(x + 1, 0, z + 1, material);
        //             contains_opaque_blocks |= material.is_opaque();
        //             contains_invisible_blocks |= !material.is_opaque();
        //         }
        //     }
        // }
        if let Some(adjecent) = world.chunk_manager.get(&adjecent[3]) {
            for x in 0..CHUNK_SIZE {
                for z in 0..CHUNK_SIZE {
                    let material = adjecent.get(x, 0, z);
                    blocks.set(x + 1, CHUNK_SIZE_SAFE - 1, z + 1, material);
                    contains_opaque_blocks |= material.is_opaque();
                    contains_invisible_blocks |= !material.is_opaque();
                }
            }
        }

        // x
        // if let Some(adjecent) = world.chunk_manager.get(&adjecent[4]) {
        //     for x in 0..CHUNK_SIZE {
        //         for y in 0..CHUNK_SIZE {
        //             let material = adjecent.get(x, y, CHUNK_SIZE - 1);
        //             blocks.set(x + 1, y + 1, 0, material);
        //             contains_opaque_blocks |= material.is_opaque();
        //             contains_invisible_blocks |= !material.is_opaque();
        //         }
        //     }
        // }
        if let Some(adjecent) = world.chunk_manager.get(&adjecent[5]) {
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    let material = adjecent.get(x, y, 0);
                    blocks.set(x + 1, y + 1, CHUNK_SIZE_SAFE - 1, material);
                    contains_opaque_blocks |= material.is_opaque();
                    contains_invisible_blocks |= !material.is_opaque();
                }
            }
        }

        if contains_opaque_blocks && !contains_invisible_blocks {
            // println!("No mesh for full chunk");
            return (None, None);
        }

        if !contains_opaque_blocks && contains_invisible_blocks {
            // println!("No mesh for empty chunk");
            return (None, None);
        }

        // remesh
        let mesh = generate_greedy_mesh(id, &blocks);
        let opaque_mesh = if mesh.vertices.is_empty() {
            None
        } else {
            Some(mesh)
        };

        let mesh = generate_greedy_mesh_water(id, &blocks);
        let water_mesh = if mesh.vertices.is_empty() {
            None
        } else {
            Some(mesh)
        };

        if opaque_mesh.is_none() && water_mesh.is_none() {
            log!(*LOG_WORLD, "[WARN] Empty mesh produced for {:?}", id);
        }

        (opaque_mesh, water_mesh)
    } else {
        println!("WARN: attempted to mesh non-exitent chunk");
        (None, None)
    }
}
