#![allow(unused)]
#![feature(test)]

extern crate nalgebra_glm as glm;
extern crate test;

use anyhow::{anyhow, Result};
use gamedata::material::Material;
use gamestate::GameState;
use graphics::{
    camera::{self, Camera, FlyingCamera},
    Mesh, Vertex,
};
use log::*;
use player::Player;
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    f32::consts::PI,
    ffi::CStr,
    fs::File,
    mem::size_of,
    os::raw::c_void,
    ptr::copy_nonoverlapping as memcpy,
    sync::{
        mpsc::{channel, Receiver},
        Arc, Condvar, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
    usize,
};
use vk_util::{app::App, buffer::create_chunk_buffers, constants::LOG_WORLD};
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_0::*,
    vk::ExtDebugUtilsExtension,
    vk::KhrSurfaceExtension,
    vk::KhrSwapchainExtension,
    window as vk_window,
};
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use world::{
    chunk_generator::ChunkGenerator, chunk_id::ChunkId, mesh_generator::generate_greedy_mesh,
    ChunkData, World, CHUNK_SIZE, CHUNK_SIZE_SAFE_CUBED,
};

mod gamestate;
pub mod models;
mod player;
pub mod render;
pub mod systems;

#[derive(Clone)]
struct GeneratedChunk(ChunkId, ChunkData, Option<Mesh>);

fn generate_chunk(id: &ChunkId) -> GeneratedChunk {
    let generator = ChunkGenerator::new();
    let (data, block_count) = generator.generate(&id);
    let compact_data = generator.compress(&data);
    let mesh = if block_count > 0 && block_count < CHUNK_SIZE_SAFE_CUBED {
        let maybe_mesh = generate_greedy_mesh(&id, &data);
        if maybe_mesh.vertices.len() == 0 {
            None
        } else {
            Some(maybe_mesh)
        }
    } else {
        None
    };

    // println!("{LOG_WORLD} Generating chunk");

    GeneratedChunk(id.clone(), compact_data, mesh)
}

pub struct WorkingQueue<I, O>
where
    I: 'static + Clone + Send,
    O: 'static + Clone + Send,
{
    name: String,
    _threads: Vec<JoinHandle<()>>,
    sender: Arc<(Mutex<VecDeque<I>>, Condvar)>,
    receiver: Receiver<O>,
}

impl<I, O> WorkingQueue<I, O>
where
    I: 'static + Clone + Send,
    O: 'static + Clone + Send,
{
    pub fn new(name: String, work_function: fn(&I) -> O) -> Self {
        let work_queue = Arc::new((Mutex::new(VecDeque::<I>::new()), Condvar::new()));
        let (result_sender, result_receiver) = channel::<O>();

        let mut threads = Vec::new();
        for thread_id in 0..8 {
            let receiver = work_queue.clone();
            let sender = result_sender.clone();

            let thread = thread::Builder::new()
                .name(format!("{name}_{thread_id}"))
                .stack_size(8 * 1024 * 1024)
                .spawn(move || loop {
                    // get task
                    let task = {
                        let (lock, condition) = &*receiver;
                        let mut queue = lock.lock().unwrap();

                        while queue.is_empty() {
                            queue = condition.wait(queue).unwrap()
                        }

                        queue.pop_front().unwrap()
                    };

                    // do work
                    let result = work_function(&task);

                    // submit result
                    match sender.send(result) {
                        Ok(_) => (),
                        Err(_) => {
                            break;
                        }
                    }

                    thread::yield_now();
                })
                .expect("Failed to spawn thread");
            threads.push(thread)
        }

        Self {
            name,
            _threads: threads,
            sender: work_queue,
            receiver: result_receiver,
        }
    }
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
        // read camera
        let direction = self.camera.movement.velocity.normalize();

        // modify camera
        self.camera.cam.position += self.camera.movement.velocity * *delta_time;
        if (self.camera.input.is_pressed()) {
            let acceleration = self.camera.cam.get_base_change_mat()
                * self.camera.input.get_as_vec()
                * self.camera.movement.acceleration_factor
                * 0.3;

            self.camera.movement.velocity += acceleration;

            if self.camera.movement.velocity.magnitude() > 20. {
                self.camera.movement.velocity.set_magnitude(20.)
            }
        } else {
            self.camera.movement.velocity = self
                .camera
                .movement
                .velocity
                .lerp(&glm::Vec3::default(), 0.2);
        }

        self.camera.cam.position += self.camera.movement.velocity * *delta_time;

        // check intersection

        if self
            .world
            .intersects_point((self.camera.cam.position + direction * 1.).into())
            || self
                .world
                .intersects_point((self.camera.cam.position + direction * 0.5).into())
        {
            // move back
            self.camera.cam.position -= self.camera.movement.velocity * *delta_time;
        }
    }

    pub fn run(mut self) -> Result<()> {
        pretty_env_logger::init();

        let generation_queue = WorkingQueue::new("chunk_generation".to_string(), generate_chunk);

        // main loop
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Game")
            .with_inner_size(LogicalSize::new(1024, 768))
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

        let mut frame_counter = 0;
        let start = Instant::now();
        let mut seconds_since_start: u64 = 0;

        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            match event {
                Event::MainEventsCleared if !destroying && !minimized => {
                    let current_frame_start = Instant::now();
                    frame_counter += 1;

                    let delta_time = &current_frame_start
                        .duration_since(previous_frame_start)
                        .as_secs_f32();

                    self.update_camera(delta_time);
                    self.load_chunks(&generation_queue);

                    if let Ok(generated_chunk) = generation_queue.receiver.try_recv() {
                        let GeneratedChunk(id, chunk_data, chunk_mesh) = generated_chunk;
                        add_chunk(&id, chunk_data, chunk_mesh, &mut self.world, &mut app);
                    }

                    previous_frame_start = current_frame_start;

                    let new_seconds_since_start =
                        &current_frame_start.duration_since(start).as_secs();

                    if *new_seconds_since_start > seconds_since_start {
                        println!("FPS: {frame_counter}");
                        frame_counter = 0;
                        seconds_since_start = *new_seconds_since_start;
                    }

                    unsafe { app.render(&window, &self.world, &self.camera) }.unwrap()
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(size),
                    ..
                } => {
                    if size.width == 0 || size.height == 0 {
                        minimized = true;
                    } else {
                        minimized = false;
                        app.resized = true;
                    }
                }
                // Destroy our Vulkan app.
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    destroying = true;
                    *control_flow = ControlFlow::Exit;
                    unsafe {
                        app.destroy();
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    if !focused && state == ElementState::Released && button == MouseButton::Left {
                        focused = true;
                        grabbed = true;
                        cursor_visible = false;
                        window.set_cursor_grab(true).expect("Cursor lock failed");
                        window.set_cursor_visible(false);
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::Focused(new_focus),
                    ..
                } => {
                    focused = new_focus;
                    grabbed = new_focus;
                    cursor_visible = !new_focus;
                    window
                        .set_cursor_grab(new_focus)
                        .expect("Cursor lock failed");
                    window.set_cursor_visible(!new_focus);
                }
                Event::DeviceEvent {
                    event:
                        DeviceEvent::Key(KeyboardInput {
                            state,
                            virtual_keycode,
                            ..
                        }),
                    ..
                } => {
                    let pressed = state == ElementState::Pressed;

                    if pressed {
                        match virtual_keycode {
                            Some(VirtualKeyCode::F1) => {
                                unsafe { app.recreate_swapchain(&window) };
                            }
                            _ => {}
                        }
                    }

                    if let Some(key) = virtual_keycode {
                        self.camera.input.set_key(key, pressed);
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { input, .. },
                    ..
                } => {
                    // gain focus & enable FPS controls
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => {
                            grabbed = false;
                            cursor_visible = true;
                            window.set_cursor_grab(false).expect("Cursor lock failed");
                            window.set_cursor_visible(true);
                        }
                        _ => (),
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
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
            }
        });
    }

    fn load_chunks(&mut self, generators: &WorkingQueue<ChunkId, GeneratedChunk>) {
        // load chunks
        let player_chunk = ChunkId::new(
            self.camera.cam.position.x as i32 / CHUNK_SIZE as i32,
            self.camera.cam.position.y as i32 / CHUNK_SIZE as i32,
            self.camera.cam.position.z as i32 / CHUNK_SIZE as i32,
        );

        let render_dist = 2;
        let mut ids = vec![];
        for z in -render_dist..=render_dist {
            for y in -render_dist..=render_dist {
                for x in -render_dist..=render_dist {
                    let chunk_id =
                        ChunkId::new(player_chunk.x + x, player_chunk.y + y, player_chunk.z + z);

                    if !self.world.chunk_manager.ids.contains(&chunk_id)
                    // && chunk_id == ChunkId::new(0, 0, 0)
                    {
                        ids.push(chunk_id);
                    }
                }
            }
        }

        ids.sort_by(|a, b| ChunkId::dist2(a, &player_chunk).cmp(&ChunkId::dist2(b, &player_chunk)));

        {
            let (lock, condition) = &*generators.sender;
            if let Ok(ref mut mutex) = lock.try_lock() {
                self.world.chunk_manager.ids.extend(ids.clone());
                mutex.extend(ids);
                condition.notify_all();
            }
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
        unsafe {
            create_chunk_buffers(&app.instance, &app.device, &id, &mesh, &mut app.data);
        };

        world.mesh_manager.insert(&id, mesh);
    }
}
