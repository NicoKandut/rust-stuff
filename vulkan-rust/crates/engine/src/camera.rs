use std::f32::consts::PI;

use gamedata::vector::Vec3;
use nalgebra_glm as glm;

use crate::input::MovementInput;

const PI_HALF: f32 = PI / 2.;

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Camera {
    pub fn up(&self) -> glm::Vec3 {
        glm::vec3(0.0, 0.0, 1.0)
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
        self.pitch += p * PI / 180.0;
        self.pitch = self.pitch.clamp(-PI_HALF + 0.01, PI_HALF - 0.01);
    }

    pub fn add_yaw(&mut self, y: f32) {
        self.yaw += y * PI / 180.0;
    }
}

#[derive(Clone, Debug)]
pub struct MovingCamera {
    pub cam: Camera,
    pub vel: f32,
    pub input: MovementInput,
}

impl MovingCamera {
    pub fn forward(&self) -> Vec3 {
        glm::vec3(self.cam.yaw.cos(), self.cam.yaw.sin(), 0.0)
    }

    pub fn right(&self) -> Vec3 {
        glm::vec3(
            (self.cam.yaw - PI / 2.).cos(),
            (self.cam.yaw - PI / 2.).sin(),
            0.0,
        )
    }

    pub(crate) fn movement(&self) -> glm::Vec3 {
        let result: glm::Vec3 = (self.input.dir[0] - self.input.dir[1]) as f32 * self.forward()
            + (self.input.dir[2] - self.input.dir[3]) as f32 * self.right()
            + (self.input.dir[4] - self.input.dir[5]) as f32 * self.cam.up();

        result * self.vel
    }
}
