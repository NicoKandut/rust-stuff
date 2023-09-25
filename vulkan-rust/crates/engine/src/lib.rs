#![feature(test)]

extern crate nalgebra_glm as glm;
extern crate test;

use anyhow::Result;
use gamedata::material::Material;
use graphics::{camera::FlyingCamera, Mesh, Ray};
use render::render_distance::calculate_chunk_diff;
use threadpool::ThreadPool;

use std::{
    sync::mpsc::{self},
    time::Instant,
};
use vk_util::{app::App, buffer::create_chunk_buffers};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use world::{
    gen::chunk::{compress, Chunk},
    mesh_generator::generate_greedy_mesh,
    slice::CubeSlice,
    traits::{Data3D, Generate, Voxelize},
    ChunkData, ChunkId, PositionalSeed, Raycast, World, WorldPosition, WorldSeed, CHUNK_SIZE,
    CHUNK_SIZE_I, CHUNK_SIZE_SAFE, CHUNK_SIZE_SAFE_CUBED,
};

pub mod models;
pub mod render;
pub mod systems;

#[derive(Clone)]
struct GeneratedChunk(ChunkId, ChunkData, Option<Mesh>);

const RENDER_DISTANCE: i32 = CHUNK_SIZE_I * 4;
const UNRENDER_DISTANCE: i32 = CHUNK_SIZE_I * 5;
const PLAYER_BUILDING_REACH: f32 = 10.0;

fn generate_chunk(world_seed: &WorldSeed, chunk_id: &ChunkId) -> GeneratedChunk {
    let chunk_seed = PositionalSeed::for_chunk(world_seed, chunk_id);
    let chunk = Chunk::generate(chunk_seed);
    let voxel_data = chunk.voxelize();
    let compact_data = compress(&voxel_data.voxels);

    // TODO: a better check to determine if a mesh. Some edgecases here. Would safe the inner if.
    let mesh = if voxel_data.voxel_count > 0 && voxel_data.voxel_count < CHUNK_SIZE_SAFE_CUBED {
        let maybe_mesh = generate_greedy_mesh(chunk_id, &voxel_data.voxels);
        if maybe_mesh.vertices.len() == 0 {
            None
        } else {
            Some(maybe_mesh)
        }
    } else {
        None
    };

    // println!("{LOG_WORLD} Generating chunk");

    GeneratedChunk(chunk_id.clone(), compact_data, mesh)
}

pub struct Engine {
    camera: FlyingCamera,
    world: World,
}

impl Engine {
    pub fn create() -> Self {
        Self {
            camera: FlyingCamera::new(glm::vec3(0., 0., 64.)),
            world: World::new(),
        }
    }

    pub fn update_camera(&mut self, delta_time: &f32) {
        if self.camera.input.is_pressed() {
            let acceleration = self.camera.cam.get_base_change_mat()
                * self.camera.input.get_as_vec()
                * self.camera.movement.acceleration_factor
                * 0.3;

            self.camera.movement.velocity += acceleration;

            if self.camera.movement.velocity.magnitude() > 10. {
                self.camera.movement.velocity.set_magnitude(10.)
            }
        } else {
            self.camera.movement.velocity = self
                .camera
                .movement
                .velocity
                .lerp(&glm::Vec3::default(), 0.2);
        }

        // world_collision
        if self.camera.movement.velocity.norm() >= 0.01 {
            let desired_movement_amount = self.camera.movement.velocity.norm() * *delta_time;
            let direction = self.camera.movement.velocity.normalize();
            let ray = Ray::new(self.camera.cam.position, direction);
            let movement_range = 0.0..1.0;

            // let allowed_movement_amount = match self.world.cast_ray(&ray, &movement_range) {
            //     Some(distance) => desired_movement_amount.min(distance - 0.5),
            //     None => desired_movement_amount,
            // };
            let allowed_movement_amount = desired_movement_amount;

            self.camera.cam.position += direction * allowed_movement_amount;
        }
    }

    pub fn run(mut self) -> Result<()> {
        pretty_env_logger::init();

        let thread_pool = ThreadPool::new("worker_thread", 7);
        let (ready_chunks_tx, ready_chunks_rx) = mpsc::channel();

        // main loop
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Game")
            .with_inner_size(LogicalSize::new(800, 450))
            .build(&event_loop)?;

        let mut app = unsafe { App::create(&window)? };

        let mut destroying = false;
        let mut minimized = false;
        let mut focused = false;
        let mut grabbed = false;
        let mut cursor_visible = false;

        let center = PhysicalPosition::new(
            (window.inner_size().width / 2) as f64,
            (window.inner_size().height / 2) as f64,
        );

        let mut previous_frame_start = Instant::now();

        let mut _frame_counter = 0;
        let start = Instant::now();
        let mut seconds_since_start: u64 = 0;

        event_loop.run(move |event, _, control_flow| {
            // *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared if !destroying && !minimized => {
                    let current_frame_start = Instant::now();
                    _frame_counter += 1;

                    let delta_time = &current_frame_start
                        .duration_since(previous_frame_start)
                        .as_secs_f32();

                    self.update_camera(delta_time);
                    self.load_chunks(&thread_pool, ready_chunks_tx.clone(), &mut app);

                    if let Ok(generated_chunk) = ready_chunks_rx.try_recv() {
                        let GeneratedChunk(id, chunk_data, chunk_mesh) = generated_chunk;
                        add_chunk(&id, chunk_data, chunk_mesh, &mut self.world, &mut app);
                    }

                    previous_frame_start = current_frame_start;

                    let new_seconds_since_start =
                        &current_frame_start.duration_since(start).as_secs();

                    if *new_seconds_since_start > seconds_since_start {
                        println!("FPS: {_frame_counter}");
                        _frame_counter = 0;
                        seconds_since_start = *new_seconds_since_start;
                    }
                    unsafe { app.render(&window, &self.world, &self.camera) }.unwrap()
                }
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::Resized(size) => {
                        if size.width == 0 || size.height == 0 {
                            minimized = true;
                        } else {
                            minimized = false;
                            app.resized = true;
                        }
                    }
                    WindowEvent::CloseRequested => {
                        destroying = true;
                        *control_flow = ControlFlow::Exit;
                        unsafe {
                            app.destroy();
                        }
                    }
                    WindowEvent::MouseInput { state, button, .. } => {
                        if !focused
                            && state == ElementState::Released
                            && button == MouseButton::Left
                        {
                            focused = true;
                            grabbed = true;
                            cursor_visible = false;
                            window.set_cursor_grab(true).expect("Cursor lock failed");
                            window.set_cursor_visible(false);
                        } else if focused && state == ElementState::Released {
                            let correction = if button == MouseButton::Left {
                                0.01
                            } else {
                                -0.01
                            };
                            let direction = self.camera.cam.direction().normalize();
                            let origin = self.camera.cam.position;
                            let ray = Ray::new(origin, direction.normalize());
                            let building_range = 0.0..PLAYER_BUILDING_REACH;
                            if let Some(distance) = self.world.cast_ray(&ray, &building_range) {
                                let hit = ray.point_on_ray(distance + correction);
                                let pos = WorldPosition::from(&hit);

                                if button == MouseButton::Left {
                                    self.world.set_block(&pos, Material::Air);
                                } else {
                                    self.world.set_block(&pos, Material::Debug);
                                }

                                let chunk_id = ChunkId::from(&pos);
                                self.remesh_chunk(&chunk_id, &mut app);
                            }
                        }
                    }
                    WindowEvent::Focused(new_focus) => {
                        focused = new_focus;
                        grabbed = new_focus;
                        cursor_visible = !new_focus;
                        window
                            .set_cursor_grab(new_focus)
                            .expect("Cursor lock failed");
                        window.set_cursor_visible(!new_focus);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        // gain focus & enable FPS controls
                        match input.virtual_keycode {
                            Some(VirtualKeyCode::Escape) => {
                                grabbed = false;
                                cursor_visible = true;
                                focused = false;
                                window.set_cursor_grab(false).expect("Cursor lock failed");
                                window.set_cursor_visible(true);
                            }
                            _ => (),
                        }
                    }
                    WindowEvent::CursorMoved { position, .. } => {
                        if grabbed && focused && !cursor_visible {
                            let (x, y) = (center.x - position.x, center.y - position.y);
                            window
                                .set_cursor_position(center)
                                .expect("Cursor position setting failed");
                            self.camera.cam.add_pitch((y * 0.1) as f32);
                            self.camera.cam.add_yaw((x * 0.1) as f32);
                        }
                    }
                    _ => {}
                },
                Event::DeviceEvent {
                    event:
                        DeviceEvent::Key(KeyboardInput {
                            state,
                            virtual_keycode,
                            ..
                        }),
                    ..
                } => {
                    if focused {
                        let pressed = state == ElementState::Pressed;

                        if pressed {
                            match virtual_keycode {
                                Some(VirtualKeyCode::F1) => {
                                    unsafe { app.recreate_swapchain(&window) }.unwrap();
                                }
                                _ => {}
                            }
                        }

                        if let Some(key) = virtual_keycode {
                            self.camera.input.set_key(key, pressed);
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn remesh_chunk(&mut self, chunk_id: &ChunkId, app: &mut App) {
        if let Some(chunk_data) = self.world.chunk_manager.get_data(&chunk_id) {
            let mut blocks = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();

            // center chunk
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(x + 1, y + 1, z + 1, chunk_data.get(x, y, z));
                    }
                }
            }

            let adjecent_chunks = [
                ChunkId::new(chunk_id.x - 1, chunk_id.y, chunk_id.z),
                ChunkId::new(chunk_id.x + 1, chunk_id.y, chunk_id.z),
                ChunkId::new(chunk_id.x, chunk_id.y - 1, chunk_id.z),
                ChunkId::new(chunk_id.x, chunk_id.y + 1, chunk_id.z),
                ChunkId::new(chunk_id.x, chunk_id.y, chunk_id.z - 1),
                ChunkId::new(chunk_id.x, chunk_id.y, chunk_id.z + 1),
            ];

            // x
            if let Some(adjecent) = self.world.chunk_manager.get_data(&adjecent_chunks[0]) {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(0, y + 1, z + 1, adjecent.get(CHUNK_SIZE - 1, y, z));
                    }
                }
            }
            if let Some(adjecent) = self.world.chunk_manager.get_data(&adjecent_chunks[1]) {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(CHUNK_SIZE_SAFE - 1, y + 1, z + 1, adjecent.get(0, y, z));
                    }
                }
            }

            // y
            if let Some(adjecent) = self.world.chunk_manager.get_data(&adjecent_chunks[2]) {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(x + 1, 0, z + 1, adjecent.get(x, CHUNK_SIZE - 1, z));
                    }
                }
            }
            if let Some(adjecent) = self.world.chunk_manager.get_data(&adjecent_chunks[3]) {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(x + 1, CHUNK_SIZE_SAFE - 1, z + 1, adjecent.get(x, 0, z));
                    }
                }
            }

            // x
            if let Some(adjecent) = self.world.chunk_manager.get_data(&adjecent_chunks[4]) {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        blocks.set(x + 1, y + 1, 0, adjecent.get(x, y, CHUNK_SIZE - 1));
                    }
                }
            }
            if let Some(adjecent) = self.world.chunk_manager.get_data(&adjecent_chunks[5]) {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        blocks.set(x + 1, y + 1, CHUNK_SIZE_SAFE - 1, adjecent.get(x, y, 0));
                    }
                }
            }

            let mesh = generate_greedy_mesh(&chunk_id, &blocks);
            if !mesh.indices.is_empty() && !mesh.vertices.is_empty() {
                add_mesh(app, &chunk_id, mesh, &mut self.world)
            } else {
                println!("WARNING: empty mesh after remeshing");
            }
        }
    }

    fn load_chunks(
        &mut self,
        thread_pool: &ThreadPool,
        parking_lot: mpsc::Sender<GeneratedChunk>,
        app: &mut App,
    ) {
        let center = WorldPosition::from(&self.camera.cam.position);
        let chunk_load = calculate_chunk_diff(
            self.world.chunk_manager.ids(),
            &center,
            RENDER_DISTANCE,
            UNRENDER_DISTANCE,
        );

        for id in chunk_load.add.iter() {
            self.world.chunk_manager.set_requested(id);
        }

        for chunk_id in chunk_load.add {
            let tx = parking_lot.clone();
            let seed = self.world.seed.clone();
            thread_pool.execute(move || {
                let chunk = generate_chunk(&seed, &chunk_id);
                tx.send(chunk).unwrap();
            });
        }

        for chunk_id in chunk_load.remove {
            unsafe { app.unload_single_chunk(&chunk_id) }
            self.world.mesh_manager.remove(&chunk_id);
            self.world.chunk_manager.remove(&chunk_id);
        }
    }
}

fn add_chunk(
    id: &ChunkId,
    chunk_data: ChunkData,
    chunk_mesh: Option<Mesh>,
    world: &mut World,
    app: &mut App,
) {
    world.chunk_manager.insert_data(&id, chunk_data);
    if let Some(mesh) = chunk_mesh {
        add_mesh(app, id, mesh, world);
    }
}

fn add_mesh(app: &mut App, id: &ChunkId, mesh: Mesh, world: &mut World) {
    unsafe {
        create_chunk_buffers(&app.instance, &app.device, &id, &mesh, &mut app.data).unwrap();
    };

    world.mesh_manager.insert(&id, mesh);
}
