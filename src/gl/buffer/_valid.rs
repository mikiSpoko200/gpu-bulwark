use super::target;
use crate::glsl;
use crate::valid;

#[hi::marker]
pub trait ForBuffer<T>: glsl::bounds::TransparentType
where
    T: target::Target
{ }

#[hi::marker]
trait ForBufferDisjointHelper<Target, Subtype> where Target: target::Target, Subtype: valid::Subtype { }

impl<T> ForBufferDisjointHelper<target::Array, valid::Scalar> for T where T: glsl::bounds::ScalarType { }
impl<T> ForBufferDisjointHelper<target::Array, valid::Vector> for T where T: glsl::bounds::VectorType { }

impl<T> ForBuffer<target::Array> for T where T: glsl::bounds::TransparentType + ForBufferDisjointHelper<target::Array, T::Subtype> { }

fn test<T>() where T: ForBuffer<target::Array> { }

fn asd() {
    test::<f32>();
    test::<glsl::Vec3>();
    test::<glsl::Mat2>();
}