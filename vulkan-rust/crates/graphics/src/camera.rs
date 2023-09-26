use nalgebra_glm as glm;
use std::f32::consts::PI;

use crate::input::FlyingMovementInput;

#[derive(Debug)]
pub struct Camera {
    pub position: glm::Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    const PI_HALF: f32 = PI / 2.;
    const DEG_TO_RAD_FACTOR: f32 = PI / 180.0;
    const MIN_PITCH: f32 = -Self::PI_HALF + 0.0001;
    const MAX_PITCH: f32 = Self::PI_HALF - 0.0001;

    pub fn up(&self) -> glm::Vec3 {
        glm::vec3(0., 0., 1.)
    }

    pub fn forward(&self) -> glm::Vec3 {
        glm::vec3(self.yaw.cos(), self.yaw.sin(), 0.)
    }

    pub fn right(&self) -> glm::Vec3 {
        glm::vec3(
            (self.yaw - Self::PI_HALF).cos(),
            (self.yaw - Self::PI_HALF).sin(),
            0.,
        )
    }

    pub fn direction(&self) -> glm::Vec3 {
        glm::vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
        )
    }
    pub fn look_at(&self) -> glm::Mat4 {
        glm::look_at(
            &self.position,
            &(self.position + self.direction()),
            &self.up(),
        )
    }

    pub fn add_pitch(&mut self, p: f32) {
        self.pitch += p * Self::DEG_TO_RAD_FACTOR;
        self.pitch = self.pitch.clamp(Self::MIN_PITCH, Self::MAX_PITCH);
    }

    pub fn add_yaw(&mut self, y: f32) {
        self.yaw += y * Self::DEG_TO_RAD_FACTOR;
    }

    pub fn get_base_change_mat(&self) -> glm::Mat3 {
        let f = self.forward();
        let r = self.right();
        let u = self.up();

        glm::Mat3::from_columns(&[f, r, u])
    }
}

#[derive(Debug)]
pub struct FlyingCamera {
    pub cam: Camera,
    pub input: FlyingMovementInput,
    pub movement: Movement,
}

#[derive(Debug)]
pub struct Movement {
    pub velocity: glm::Vec3,
    pub max_velocity: f32,
    pub acceleration_factor: f32,
}

impl FlyingCamera {
    pub fn new(position: glm::Vec3) -> Self {
        Self {
            cam: Camera {
                position,
                pitch: 0.,
                yaw: 0.,
            },
            movement: Movement {
                velocity: glm::Vec3::default(),
                max_velocity: 10.,
                acceleration_factor: 3.,
            },
            input: FlyingMovementInput::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MovementInput {
    pub dir: [isize; 6],
}
