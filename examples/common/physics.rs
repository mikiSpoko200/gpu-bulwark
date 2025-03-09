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

    pub const fn new(position: glm::Vec3) -> Self {
        Self {
            velocity: glm::Vec3::default(),
            position,
        }
    }
}

pub use foundation::*;

pub mod motion {
    use super::Kinetic;
    use nalgebra_glm as glm;
    use std::time::Duration;

    // Trait for different motion models
    pub trait Model {
        fn update(&self, dt: Duration);
    }

    // 1. Exponential Decay (First-Order Inertia)
    pub struct ExponentialDecay<K: Kinetic> {
        decay_rate: f32,
        kinetic: K,
    }

    impl<K: Kinetic> Model for ExponentialDecay<K> {
        fn update(&self, dt: Duration) {
            let velocity = *self.kinetic.velocity()
                * (-self.decay_rate * dt.as_micros() as f32 / 1_000_000.0).exp();
            *obj.velocity() = velocity;
            *obj.position() += velocity * dt;
        }
    }

    // 2. Critically Damped Spring (Smooth Target Following)
    pub struct CriticallyDampedSpring {
        target: glm::Vec3,
        stiffness: f32,
        damping: f32,
    }

    impl<T: Kinetic> Model<T> for CriticallyDampedSpring {
        fn update(&self, obj: &mut T, dt: f32) {
            let diff = self.target - *obj.position();
            let force = diff * self.stiffness - *obj.velocity() * self.damping;
            obj.kinetic_state().apply_acceleration(force, dt);
        }
    }

    // 3. Velocity-Based Damping (Friction Model)
    pub struct VelocityDamping {
        friction: f32,
    }

    impl<T: Kinetic> Model<T> for VelocityDamping {
        fn update(&self, obj: &mut T, dt: f32) {
            let friction_force = *obj.velocity() * -self.friction;
            obj.kinetic_state().apply_acceleration(friction_force, dt);
        }
    }

    // Main loop with fixed timestep
    type Time = f32;
    const FIXED_DT: Time = 1.0 / 120.0; // 120Hz physics rate
}
