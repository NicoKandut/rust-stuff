#![feature(test)]

extern crate nalgebra_glm as glm;
extern crate test;

use anyhow::Result;
use chunk_stream::{ChunkAction, ChunkTracker};
use gamedata::material::Material;
use geometry::Ray;
use graphics::{camera::FlyingCamera, Mesh};
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
    ChunkData, ChunkId, ChunkUpdateData, PositionalSeed, Raycast, World, WorldPosition, WorldSeed,
    CHUNK_SIZE, CHUNK_SIZE_F, CHUNK_SIZE_I, CHUNK_SIZE_SAFE, CHUNK_SIZE_SAFE_CUBED,
};

mod chunk_stream;
pub mod models;
pub mod render;
pub mod systems;

#[derive(Clone)]
struct ChunkUpdate(ChunkId, ChunkData, Option<Mesh>);

const RENDER_DISTANCE: f32 = CHUNK_SIZE_F * 4.0;
const UNRENDER_DISTANCE: f32 = CHUNK_SIZE_F * 5.0;
const PLAYER_BUILDING_REACH: f32 = 10.0;

fn generate_chunk(world_seed: &WorldSeed, chunk_id: &ChunkId) -> ChunkUpdate {
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

    ChunkUpdate(chunk_id.clone(), compact_data, mesh)
}

pub struct Engine {
    camera: FlyingCamera,
    world: World,
}

pub struct BlockUpdate {
    pub position: WorldPosition,
    pub material: Material,
}

impl Engine {
    pub fn create() -> Self {
        println!("Creating engine");
        Self {
            camera: FlyingCamera::new(glm::vec3(0., 0., 64.)),
            world: World::new(),
        }
    }

    pub fn update_camera(&mut self, delta_time: &f32, godmode: bool) {
        if self.camera.input.is_pressed() {
            let acceleration = self.camera.cam.get_base_change_mat()
                * self.camera.input.get_as_vec()
                * self.camera.movement.acceleration_factor
                * 0.3;

            self.camera.movement.velocity += acceleration;

            let max_velocity = if godmode {
                self.camera.movement.max_velocity * 10.0
            } else {
                self.camera.movement.max_velocity
            };

            if self.camera.movement.velocity.norm() > max_velocity {
                self.camera.movement.velocity.set_magnitude(max_velocity)
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

            let allowed_movement_amount = if godmode {
                desired_movement_amount
            } else {
                match self.world.cast_ray(&ray, &movement_range) {
                    Some(distance) => desired_movement_amount.min(distance - 0.5),
                    None => desired_movement_amount,
                }
            };

            self.camera.cam.position += direction * allowed_movement_amount;
        }
    }

    pub fn run(mut self) -> Result<()> {
        println!("Running engine");
        pretty_env_logger::init();

        println!("Setting up multithreading");
        let thread_pool = ThreadPool::new("worker_thread", 7);
        let (chunk_update_tx, chunk_update_rx) = mpsc::channel();
        let mut chunk_stream = ChunkTracker::new(RENDER_DISTANCE, UNRENDER_DISTANCE);

        println!("Setting up window");
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Game")
            .with_inner_size(LogicalSize::new(800, 450))
            .build(&event_loop)?;

        println!("Setting up app");
        let mut app = unsafe { App::create(&window)? };

        println!("Setting up state");
        let mut destroying = false;
        let mut minimized = false;
        let mut focused = false;
        let mut grabbed = false;
        let mut cursor_visible = false;
        let mut godmode = false;

        let center = PhysicalPosition::new(
            (window.inner_size().width / 2) as f64,
            (window.inner_size().height / 2) as f64,
        );

        let mut previous_frame_start = Instant::now();

        let mut _frame_counter = 0;
        let start = Instant::now();
        let mut seconds_since_start: u64 = 0;

        println!("Entering event loop");

        event_loop.run(move |event, _, control_flow| {
            // *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared if !destroying && !minimized => {
                    let current_frame_start = Instant::now();
                    _frame_counter += 1;

                    let delta_time = &current_frame_start
                        .duration_since(previous_frame_start)
                        .as_secs_f32();

                    self.update_camera(delta_time, godmode);
                    self.queue_chunk_changes(
                        &thread_pool,
                        chunk_update_tx.clone(),
                        &mut app,
                        &mut chunk_stream,
                    );

                    while let Ok(generated_chunk) = chunk_update_rx.try_recv() {
                        let ChunkUpdate(id, chunk_data, chunk_mesh) = generated_chunk;
                        add_chunk_data(&id, chunk_data, &mut self.world);
                        if let Some(mesh) = chunk_mesh {
                            add_chunk_mesh(&mut app, &id, mesh, &mut self.world)
                        }
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
                                let position = WorldPosition::from(&hit);
                                let chunk_id = ChunkId::from(&position);
                                let material = if button == MouseButton::Left {
                                    Material::Air
                                } else {
                                    Material::Debug
                                };

                                let update = BlockUpdate { position, material };
                                let update_data =
                                    self.world.chunk_manager.get_update_data(&chunk_id);

                                let chunk_update_tx = chunk_update_tx.clone();
                                thread_pool.execute(move || {
                                    if let Some(update) = Self::update_chunk(update, update_data) {
                                        chunk_update_tx.send(update).unwrap()
                                    }
                                })
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
                                Some(VirtualKeyCode::Tab) => {
                                    godmode = !godmode;
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

    fn update_chunk(update: BlockUpdate, context: ChunkUpdateData) -> Option<ChunkUpdate> {
        // read old data
        if let Some(chunk_data) = context.chunk.data {
            let mut blocks = CubeSlice::<Material, CHUNK_SIZE_SAFE>::default();

            // center chunk
            for x in 0..CHUNK_SIZE {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(x + 1, y + 1, z + 1, chunk_data.get(x, y, z));
                    }
                }
            }

            // x
            if let Some(adjecent) = &context.adjecent[0].data {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(0, y + 1, z + 1, adjecent.get(CHUNK_SIZE - 1, y, z));
                    }
                }
            }
            if let Some(adjecent) = &context.adjecent[1].data {
                for y in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(CHUNK_SIZE_SAFE - 1, y + 1, z + 1, adjecent.get(0, y, z));
                    }
                }
            }

            // y
            if let Some(adjecent) = &context.adjecent[2].data {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(x + 1, 0, z + 1, adjecent.get(x, CHUNK_SIZE - 1, z));
                    }
                }
            }
            if let Some(adjecent) = &context.adjecent[3].data {
                for x in 0..CHUNK_SIZE {
                    for z in 0..CHUNK_SIZE {
                        blocks.set(x + 1, CHUNK_SIZE_SAFE - 1, z + 1, adjecent.get(x, 0, z));
                    }
                }
            }

            // x
            if let Some(adjecent) = &context.adjecent[4].data {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        blocks.set(x + 1, y + 1, 0, adjecent.get(x, y, CHUNK_SIZE - 1));
                    }
                }
            }
            if let Some(adjecent) = &context.adjecent[5].data {
                for x in 0..CHUNK_SIZE {
                    for y in 0..CHUNK_SIZE {
                        blocks.set(x + 1, y + 1, CHUNK_SIZE_SAFE - 1, adjecent.get(x, y, 0));
                    }
                }
            }

            let pos_in_chunk = update.position.rem_euclid(CHUNK_SIZE_I);
            blocks.set(
                1 + pos_in_chunk.x as usize,
                1 + pos_in_chunk.y as usize,
                1 + pos_in_chunk.z as usize,
                update.material,
            );

            let mesh = generate_greedy_mesh(&context.chunk.id, &blocks);
            let opt_mesh = if !mesh.indices.is_empty() && !mesh.vertices.is_empty() {
                Some(mesh)
            } else {
                println!("WARNING: empty mesh after remeshing");
                None
            };
            let data = compress(&blocks);

            return Some(ChunkUpdate(context.chunk.id, data, opt_mesh));
        }

        None
    }

    fn queue_chunk_changes(
        &mut self,
        thread_pool: &ThreadPool,
        new_chunks: mpsc::Sender<ChunkUpdate>,
        app: &mut App,
        chunk_stream: &mut ChunkTracker,
    ) {
        let actions = chunk_stream.update(&self.camera.cam.position);

        if !actions.is_empty() {
            println!("{} actions this frame", actions.len());
        }

        for action in actions {
            let new_chunks = new_chunks.clone();
            match action {
                ChunkAction::Load(id) => self.load_chunk(id, thread_pool, new_chunks),
                ChunkAction::Unload(id) => self.unload_chunk(id, app),
                // ChunkAction::Update(id) => self.update_chunk(id),
            }
        }
    }

    fn unload_chunk(&mut self, chunk_id: ChunkId, app: &mut App) {
        unsafe { app.unload_single_chunk(&chunk_id) }
        self.world.mesh_manager.remove(&chunk_id);
        self.world.chunk_manager.remove(&chunk_id);
    }

    fn load_chunk(
        &mut self,
        chunk_id: ChunkId,
        thread_pool: &ThreadPool,
        new_chunks: mpsc::Sender<ChunkUpdate>,
    ) {
        self.world.chunk_manager.set_requested(&chunk_id);
        let seed = self.world.seed.clone();
        thread_pool.execute(move || {
            let chunk = generate_chunk(&seed, &chunk_id);
            new_chunks.send(chunk).unwrap();
        });
    }
}

fn add_chunk_data(id: &ChunkId, chunk_data: ChunkData, world: &mut World) {
    world.chunk_manager.insert_data(&id, chunk_data);
}

fn add_chunk_mesh(app: &mut App, id: &ChunkId, mesh: Mesh, world: &mut World) {
    unsafe {
        create_chunk_buffers(&app.instance, &app.device, &id, &mesh, &mut app.data).unwrap();
    };

    world.mesh_manager.insert(&id, mesh);
}

// fn remove_chunk_mesh(app: &mut App, id: &ChunkId, world: &mut World) {
//     unsafe { app.unload_single_chunk(&id) }
//     world.mesh_manager.remove(&id);
// }
