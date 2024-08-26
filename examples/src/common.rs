#![allow(unused)]

pub mod config {
    pub struct Config {
        pub width: u32,
        pub height: u32,
    }

    impl Default for Config {
        fn default() -> Self {
            Self { width: WIDTH, height: HEIGHT }
        }
    }

    pub const WIDTH: u32 = 960;
    pub const HEIGHT: u32 = 640;
    pub const MOUSE_SENSITIVITY: f32 = 0.005;
    pub const MOVEMENT_SPEED: f32 = 0.1;
}


pub mod camera {
    use glm::Vec3;

    const MOVEMENT_SPEED: f32 = 0.1;
    const MOUSE_SENSITIVITY: f32 = 0.005;

    pub struct Camera {
        pub position: Vec3,
        pub yaw: f32,
        pub pitch: f32,
        aspect_ratio: f32,
    }

    impl Default for Camera {
        fn default() -> Self {
            Self { position: Default::default(), yaw: Default::default(), pitch: Default::default(), aspect_ratio: 1.33 }
        }
    }

    impl Camera {
        pub fn new(position: Vec3, yaw: f32, pitch: f32, aspect_ratio: f32) -> Self {
            Camera {
                position,
                yaw,
                pitch,
                aspect_ratio,
            }
        }

        fn view_matrix(&self) -> glm::Mat4 {
            let front = Vec3::new(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            );
            glm::look_at_rh(&self.position, &(self.position + front), &Vec3::y())
        }

        fn projection_matrix(&self) -> glm::Mat4 {
            glm::perspective_rh_no(
                45.0 * std::f32::consts::FRAC_1_PI * 0.5,
                self.aspect_ratio,
                0.1,
                1000.0,
            )
        }

        pub fn view_projection_matrix(&self) -> glm::Mat4 {
            self.projection_matrix() * self.view_matrix()
        }

        pub fn process_input(&mut self, input: &winit::keyboard::KeyCode) {
            match input {
                winit::keyboard::KeyCode::KeyW => Some(Vec3::new( 0.0,  0.0, -1.0)),
                winit::keyboard::KeyCode::KeyS => Some(Vec3::new( 0.0,  0.0,  1.0)),
                winit::keyboard::KeyCode::KeyA => Some(Vec3::new(-1.0,  0.0,  0.0)),
                winit::keyboard::KeyCode::KeyD => Some(Vec3::new( 1.0,  0.0,  0.0)),
                _ => None,
            }
            .inspect(|movement| {
                let rotation_matrix = glm::rotation(self.yaw, &Vec3::y());
                self.position += (rotation_matrix
                    * glm::vec4(movement.x, movement.y, movement.z, 1.0)
                    * MOVEMENT_SPEED)
                    .xyz();
            });
        }

        pub fn process_mouse(&mut self, dx: f64, dy: f64) {
            self.yaw += dx as f32 * MOUSE_SENSITIVITY;
            self.pitch += dy as f32 * MOUSE_SENSITIVITY;

            if self.pitch > 1.5 {
                self.pitch = 1.5;
            }
            if self.pitch < -1.5 {
                self.pitch = -1.5;
            }
        }
    }
}
