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
            self.velocity += acceleration * dt;
            self.position += self.velocity * dt;
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

            // Create rotation quaternions for each axis
            let qx = glm::quat_rotation(&glm::Vec3::x(), x_rad);
            let qy = glm::quat_rotation(&glm::Vec3::y(), y_rad);

            // Apply the rotations: first rotate around Y, then X
            let rotation = qy * qx;

            // Apply the rotation to the facing direction
            self.facing_dir = rotation * self.facing_dir;
        }
    }

    pub trait BasicRotate {
        fn basic_rotate(&mut self) -> &mut BasicRotateState;
    }
}

pub use foundation::*;

pub mod motion {
    use std::time::Instant;
    use nalgebra_glm::Vec3;
    use super::*;
    use std::ops::{Mul, Add, Div, Sub};

    // Trait for different motion models
    trait KineticModel {
        fn apply(&self, dt: f32);
    }

    // 1. Exponential Decay (First-Order Inertia)
    struct ExponentialDecay<K: Kinetic> {
        decay_rate: f32,
        kinetic: K
    }

    impl<K: Kinetic> KineticModel for ExponentialDecay<K> {
        fn apply(&self, dt: f32) {
            let KineticState { mut velocity, mut position } = self.kinetic.kinetic_state();

            velocity *= (-self.decay_rate * dt).exp();
            position += velocity * dt;
        }
    }

    // 2. Critically Damped Spring (Smooth Target Following)
    struct CriticallyDampedSpring<K: Kinetic> {
        target: Vec3,
        stiffness: f32,
        damping: f32,
        kinetic: K,
    }

    impl<K: Kinetic> KineticModel for CriticallyDampedSpring<K> {
        fn apply(&self, dt: f32) {
            let KineticState { mut velocity, mut position } = self.kinetic.kinetic_state();

            let diff = self.target - position;
            let force = diff * velocity * -self.damping * self.stiffness;
            velocity += force * dt;
            position += velocity * dt;
        }
    }

    // 3. Velocity-Based Damping (Friction Model)
    struct VelocityDamping {
        friction: f32,
    }

    impl KineticModel for VelocityDamping {
        fn apply(&self, obj: &mut dyn Movable, dt: f32) {
            let friction_force = obj.velocity().scale(-self.friction);
            let new_velocity = obj.velocity().add(&friction_force.scale(dt));
            obj.set_velocity(new_velocity);
            obj.set_position(obj.position().add(&new_velocity.scale(dt)));
        }
    }

    // Main loop with fixed timestep
    type Time = f32;
    const FIXED_DT: Time = 1.0 / 120.0; // 120Hz physics rate

    fn main() {
        let mut obj = Object::new();
        let motion_model = ExponentialDecay { decay_rate: 5.0 };
        let mut last_time = Instant::now();
        let mut accumulated_time: Time = 0.0;

        loop {
            let now = Instant::now();
            let frame_time = now.duration_since(last_time).as_secs_f32();
            last_time = now;
            accumulated_time += frame_time;

            while accumulated_time >= FIXED_DT {
                motion_model.apply(&mut obj, FIXED_DT);
                accumulated_time -= FIXED_DT;
            }

            println!("Object Position: {:?}", obj.position());
        }
    }
}
