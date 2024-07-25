use super::target;
use crate::glsl;
use crate::valid;

use crate::prelude::internal::*;

#[hi::marker]
pub trait ForBuffer<T>: glsl::bounds::TransparentType
where
    T: target::Target
{ }

#[hi::marker]
trait ForBufferDH<Target, Subtype>
where 
    Target: target::Target,
    Subtype: valid::Subtype
{ }

impl<T> ForBufferDH<target::Array, valid::Scalar> for T
where
    T: glsl::bounds::ScalarType
{ }

impl<T, const DIM: usize> ForBufferDH<target::Array, valid::Vector<DIM>> for T
where 
    T: glsl::bounds::VectorType<DIM>,
    Const<DIM>: valid::VecDim,
{ }

impl<T> ForBuffer<target::Array> for T where T: glsl::bounds::TransparentType + ForBufferDH<target::Array, T::Subtype> { }

fn test<T>() where T: ForBuffer<target::Array> { }

fn asd() {
    test::<f32>();
    test::<glsl::Vec3>();
}