use nalgebra_glm as glm;

pub trait Kinematic {
    
}

pub struct KineticState {
    velocity: glm::Vec3,
    position: glm::Vec3,
}

impl KineticState {
    pub fn apply_acceleration(&mut self, acceleration: glm::Vec3, dt: f32) {
        self.velocity += acceleration * dt;
        self.position += self.velocity * dt;
    }
}

pub trait Moveable {
    fn kinetic_state(&mut self) -> &mut KineticState;

    fn velocity(&mut self) -> &mut nalgebra_glm::Vec3 {
        &mut self.kinetic_state().velocity
    }

    fn position(&mut self) -> &mut nalgebra_glm::Vec3 {
        &mut self.kinetic_state().position
    }
}
