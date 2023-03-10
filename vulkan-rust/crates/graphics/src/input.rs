use winit::event::VirtualKeyCode;

#[derive(Clone, Debug)]
pub struct FlyingMovementInput {
    up: bool,
    down: bool,
    forward: bool,
    backward: bool,
    left: bool,
    right: bool,
}

impl FlyingMovementInput {
    pub fn new() -> Self {
        Self {
            up: false,
            down: false,
            forward: false,
            backward: false,
            left: false,
            right: false,
        }
    }

    pub fn is_pressed(&self) -> bool {
        self.up || self.down || self.forward || self.backward || self.left || self.right
    }

    pub fn get_as_vec(&self) -> glm::Vec3 {
        glm::vec3(
            (self.forward as i8 - self.backward as i8) as f32,
            (self.right as i8 - self.left as i8) as f32,
            (self.up as i8 - self.down as i8) as f32,
        )
    }

    pub fn set_key(&mut self, key: VirtualKeyCode, state: bool) {
        match key {
            VirtualKeyCode::W => {
                self.forward = state;
            }
            VirtualKeyCode::S => {
                self.backward = state;
            }
            VirtualKeyCode::D => {
                self.right = state;
            }
            VirtualKeyCode::A => {
                self.left = state;
            }
            VirtualKeyCode::Space => {
                self.up = state;
            }
            VirtualKeyCode::C => {
                self.down = state;
            }
            _ => (),
        }
    }
}
