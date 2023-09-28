#![feature(test)]
#![feature(deadline_api)]

extern crate nalgebra_glm as glm;
extern crate test;

use crate::world_thread::{MeshEvent, Request};
use anyhow::Result;
use gamedata::material::Material;
use geometry::Ray;
use graphics::camera::FlyingCamera;
use logging::{log, LOG_ENGINE, LOG_RENDER};
use std::{
    collections::{HashMap, VecDeque},
    sync::mpsc,
    time::{Duration, Instant},
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
use world::{ChunkData, ChunkId, WorldPosition, CHUNK_SIZE_F};

mod chunk_stream;
pub mod models;
pub mod render;
pub mod systems;
mod world_thread;

#[derive(Clone)]
struct ChunkUpdate(ChunkId, ChunkData, Instant);

const INITIAL_RENDER_DISTANCE: f32 = CHUNK_SIZE_F * 6.0;
const INITIAL_UNRENDER_DISTANCE: f32 = CHUNK_SIZE_F * 7.0;
const PLAYER_BUILDING_REACH: f32 = 10.0;
const MESH_TIME_WINDOW_SIZE: usize = 20;

pub struct Engine {
    camera: FlyingCamera,
    meshes: HashMap<ChunkId, usize>,
}

pub struct BlockUpdate {
    pub position: WorldPosition,
    pub material: Material,
}

impl Engine {
    pub fn create() -> Self {
        log!(*LOG_ENGINE, "Creating engine");
        Self {
            camera: FlyingCamera::new(glm::vec3(0., 0., 64.)),
            meshes: HashMap::new(),
        }
    }

    pub fn run(mut self) -> Result<()> {
        log!(*LOG_ENGINE, "Running engine");
        pretty_env_logger::init();

        log!(*LOG_ENGINE, "Setting up world thread");
        let (_world_thread, world_requests, world_events) = world_thread::spawn();
        world_requests
            .send(Request::SetRenderDistance(
                INITIAL_RENDER_DISTANCE,
                INITIAL_UNRENDER_DISTANCE,
                Instant::now(),
            ))
            .expect("World Thread must be available");

        log!(*LOG_ENGINE, "Setting up window");
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_title("Game")
            .with_inner_size(LogicalSize::new(800, 450))
            .build(&event_loop)?;

        log!(*LOG_ENGINE, "Setting up app");
        let mut app = unsafe { App::create(&window)? };

        log!(*LOG_ENGINE, "Setting up state");
        let mut destroying = false;
        let mut minimized = false;
        let mut focused = false;
        let mut grabbed = false;
        let mut cursor_visible = false;
        let mut godmode = false;
        let mut render_distance = INITIAL_RENDER_DISTANCE;
        let mut unrender_distance = INITIAL_UNRENDER_DISTANCE;

        let center = PhysicalPosition::new(
            (window.inner_size().width / 2) as f64,
            (window.inner_size().height / 2) as f64,
        );

        let mut previous_frame_start = Instant::now();

        let mut frame_counter = 0;
        let start = Instant::now();
        let mut seconds_since_start: u64 = 0;

        let mut last_sent_position = glm::vec3(0.0, 0.0, 0.0);
        let mut last_sent_time = start;

        let mut mesh_times = VecDeque::with_capacity(MESH_TIME_WINDOW_SIZE);

        println!("Entering event loop");

        event_loop.run(move |event, _, control_flow| {
            // *control_flow = ControlFlow::Poll;

            match event {
                Event::MainEventsCleared if !destroying && !minimized => {
                    let current_frame_start = Instant::now();
                    let delta_time = &current_frame_start
                        .duration_since(previous_frame_start)
                        .as_secs_f32();

                    self.update_camera(delta_time, godmode);
                    self.send_world_thread_move_request(
                        &world_requests,
                        &mut last_sent_time,
                        &mut last_sent_position,
                    );
                    self.receive_mesh_events(
                        &world_events,
                        &mut app,
                        &mut mesh_times,
                        &current_frame_start,
                    );

                    unsafe { app.render(&window, &self.meshes, &self.camera) }.unwrap();

                    log_stats(
                        start,
                        current_frame_start,
                        &mut seconds_since_start,
                        &mut frame_counter,
                        &mut mesh_times,
                    );

                    frame_counter += 1;
                    previous_frame_start = current_frame_start;
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
                        world_requests
                            .send(world_thread::Request::Exit)
                            .expect("World Thread must be available");
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
                            let ray = Ray::new(
                                self.camera.cam.position,
                                self.camera.cam.direction().normalize(),
                            );

                            match button {
                                MouseButton::Left => {
                                    world_requests
                                        .send(world_thread::Request::Modify {
                                            ray: ray.clone(),
                                            range: PLAYER_BUILDING_REACH,
                                            action: world_thread::ModifyAction::Remove,
                                            start: Instant::now(),
                                        })
                                        .expect("World Thread must be available");
                                }
                                MouseButton::Right => {
                                    world_requests
                                        .send(world_thread::Request::Modify {
                                            ray: ray.clone(),
                                            range: PLAYER_BUILDING_REACH,
                                            action: world_thread::ModifyAction::Place(
                                                Material::Debug,
                                            ),
                                            start: Instant::now(),
                                        })
                                        .expect("World Thread must be available");
                                }
                                MouseButton::Middle => {}
                                _ => {}
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
                                Some(VirtualKeyCode::Up) => {
                                    render_distance += CHUNK_SIZE_F;
                                    unrender_distance += CHUNK_SIZE_F;
                                    world_requests
                                        .send(Request::SetRenderDistance(
                                            render_distance,
                                            unrender_distance,
                                            Instant::now(),
                                        ))
                                        .expect("World Thread must be available");
                                }
                                Some(VirtualKeyCode::Down) => {
                                    render_distance -= CHUNK_SIZE_F;
                                    unrender_distance -= CHUNK_SIZE_F;
                                    world_requests
                                        .send(Request::SetRenderDistance(
                                            render_distance,
                                            unrender_distance,
                                            Instant::now(),
                                        ))
                                        .expect("World Thread must be available");
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
        })

        // world_thread.join().expect("World thread should not panic");
    }

    fn receive_mesh_events(
        &mut self,
        world_events: &mpsc::Receiver<MeshEvent>,
        app: &mut App,
        mesh_times: &mut VecDeque<Duration>,
        current_frame_start: &Instant,
    ) {
        const ALLOWED_FRAME_TIME: Duration = Duration::from_millis(10);

        let mut meshes_in_creation = vec![];
        let mut meshes_in_destruction = vec![];

        while let Ok(mesh_event) = world_events.try_recv() {
            match mesh_event {
                MeshEvent::Add(id, mesh, start) => {
                    self.meshes.insert(id, mesh.indices.len());
                    meshes_in_creation.push((id, mesh));
                    mesh_times.push_back(start.elapsed());
                    if mesh_times.len() > MESH_TIME_WINDOW_SIZE {
                        mesh_times.pop_front();
                    }
                }
                MeshEvent::Remove(id) => {
                    meshes_in_destruction.push(id);
                    self.meshes.remove(&id);
                }
            }
            if current_frame_start.elapsed() > ALLOWED_FRAME_TIME {
                break;
            }
        }

        for (id, mesh) in meshes_in_creation {
            unsafe {
                create_chunk_buffers(&app.instance, &app.device, &id, &mesh, &mut app.data)
                    .unwrap();
            };
        }

        for id in meshes_in_destruction {
            unsafe { app.unload_single_chunk(&id) }
        }
    }

    fn send_world_thread_move_request(
        &self,
        world_requests: &mpsc::Sender<Request>,
        last_sent_time: &mut Instant,
        last_sent_position: &mut glm::Vec3,
    ) {
        const MIN_UPDATE_INTERVAL: Duration = Duration::from_millis(500);
        const MIN_UPDATE_DISTANCE: f32 = 5.0;

        if last_sent_time.elapsed() > MIN_UPDATE_INTERVAL
            && glm::distance(&self.camera.cam.position, &last_sent_position) > MIN_UPDATE_DISTANCE
        {
            *last_sent_position = self.camera.cam.position;
            *last_sent_time = Instant::now();

            world_requests
                .send(Request::Move(*last_sent_position, *last_sent_time))
                .expect("World Thread must be available");
        }
    }

    fn update_camera(&mut self, delta_time: &f32, godmode: bool) {
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
            // let ray = Ray::new(self.camera.cam.position, direction);
            // let movement_range = 0.0..1.0;

            let allowed_movement_amount = if godmode {
                desired_movement_amount
            } else {
                // match self.world.cast_ray(&ray, &movement_range) {
                //     Some(distance) => desired_movement_amount.min(distance - 0.5),
                //     None => desired_movement_amount,
                // }
                desired_movement_amount
            };

            self.camera.cam.position += direction * allowed_movement_amount;
        }
    }
}

fn log_stats(
    start: Instant,
    current_frame_start: Instant,
    seconds_since_start: &mut u64,
    frame_counter: &mut i32,
    mesh_times: &mut VecDeque<Duration>,
) {
    let new_seconds_since_start = current_frame_start.duration_since(start).as_secs();
    if new_seconds_since_start > *seconds_since_start {
        log!(*LOG_RENDER, "FPS: {}", frame_counter);
        log!(
            *LOG_RENDER,
            "  Avg. Mesh Time: {:?}",
            mesh_times.iter().sum::<Duration>() / mesh_times.len().max(1) as u32
        );
        *frame_counter = 0;
        *seconds_since_start = new_seconds_since_start;
    }
}
