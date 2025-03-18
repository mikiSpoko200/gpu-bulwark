use nalgebra_glm as glm;

pub mod foundation {
    use super::*;
    
    pub struct KineticState {
        pub velocity: glm::Vec3,
        pub position: glm::Vec3,
    }

    impl KineticState {
        pub fn stationary(position: glm::Vec3) -> Self {
            Self {
                position,
                velocity: glm::Vec3::default()
            }
        }

        pub fn apply_acceleration(&mut self, acceleration: glm::Vec3, dt: f32) {
            self.position += self.velocity * dt + 0.5 * acceleration * dt * dt;
            self.velocity += acceleration * dt;
        }
    }

    pub trait Kinetic {
        fn kinetic_state(&mut self) -> &mut KineticState;

        fn velocity(&mut self) -> &mut nalgebra_glm::Vec3 {
            &mut self.kinetic_state().velocity
        }

        fn position(&mut self) -> &mut nalgebra_glm::Vec3 {
            &mut self.kinetic_state().position
        }
    }

    pub struct BasicRotateState {
        facing_dir: glm::Vec3,
    }

    impl BasicRotateState {
        pub fn basic_rotate(&mut self, x_angle: f32, y_angle: f32) {
            // Convert angles from degrees to radians
            let x_rad = x_angle.to_radians();
            let y_rad = y_angle.to_radians();
    
            // Define explicit unit vectors for rotation axes
            let x_axis = glm::vec3(1.0, 0.0, 0.0);
            let y_axis = glm::vec3(0.0, 1.0, 0.0);
    
            // Create rotation quaternions
            let qx = glm::quat_angle_axis(x_rad, &x_axis);
            let qy = glm::quat_angle_axis(y_rad, &y_axis);
    
            // Apply the rotations: first rotate around Y (yaw), then X (pitch)
            let rotation = qy * qx;
    
            // Apply the rotation to the facing direction
            self.facing_dir = glm::quat_rotate_vec3(&rotation, &self.facing_dir);
        }
    }

    pub trait BasicRotate {
        fn basic_rotate(&mut self) -> &mut BasicRotateState;
    }
}

pub use foundation::*;

use super::KeyBoard;

pub mod motion {
    use super::{Kinetic, KineticState};
    use nalgebra_glm as glm;

    pub trait Model {
        fn update(&mut self, dt: f32);
    }

    pub struct ExponentialDecay<K: Kinetic> {
        decay_rate: f32,
        kinetic: K,
    }

    impl<K: Kinetic> Model for ExponentialDecay<K> {
        fn update(&mut self, dt: f32) {
            let KineticState { velocity, position } = self.kinetic.kinetic_state();
            *velocity *= (-self.decay_rate * dt).exp();
            *position += *velocity * dt;
        }
    }

    pub struct CriticallyDampedSpring<K: Kinetic> {
        target: glm::Vec3,
        stiffness: f32,
        damping: f32,
        kinetic: K,
    }

    impl<K: Kinetic> Model for CriticallyDampedSpring<K> {
        fn update(&mut self, dt: f32) {
            let KineticState { velocity, position } = self.kinetic.kinetic_state();
            let diff = self.target - *position;
            let force = diff * self.stiffness - *velocity * self.damping;
            self.kinetic.kinetic_state().apply_acceleration(force, dt);
        }
    }

    pub struct VelocityDamping<K: Kinetic> {
        friction: f32,
        kinetic: K,
    }

    impl<K: Kinetic> Model for VelocityDamping<K> {
        fn update(&mut self, dt: f32) {
            let friction_force = *self.kinetic.velocity() * -self.friction;
            self.kinetic.kinetic_state().apply_acceleration(friction_force, dt);
        }
    }
    const FIXED_DT: f32 = 1.0 / 120.0;
}

pub fn steer(input: &KeyBoard, obj: &mut impl Kinetic, dt: f32) {
    let mut target_velocity = glm::vec3(0.0, 0.0, 0.0);
    let max_speed = 10.0;

    if input.key_w.is_on() {
        target_velocity.z -= 1.0;
    }
    if input.key_s.is_on() {
        target_velocity.z += 1.0;
    }
    if input.key_a.is_on() {
        target_velocity.x -= 1.0;
    }
    if input.key_d.is_on() {
        target_velocity.x += 1.0;
    }

    // Normalize direction and scale by max speed (to avoid diagonal speed issues)
    if glm::length(&target_velocity) > 0.0 {
        target_velocity = glm::normalize(&target_velocity) * max_speed;
    }

    // Spring-based acceleration model
    let k = 10.0; // Acceleration factor
    let d = 5.0;  // Damping factor (friction)
    
    let velocity = *obj.velocity();
    let acceleration = k * (target_velocity - velocity) - d * velocity;

    obj.kinetic_state().apply_acceleration(acceleration, dt);
}
