pub trait Moveable {
    fn velocity(&mut self) -> &mut nalgebra_glm::Vec3;
    fn position(&mut self) -> &mut nalgebra_glm::Vec3;
}
