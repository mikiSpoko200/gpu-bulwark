//! This module provides specialization of HLists for Program / Shader parameters.

use crate::prelude::internal::*;

use crate::glsl;
use crate::hlist::lhlist::Base as HList;
use glsl::{InBinding, OutBinding, Storage, Qualifier, storage};


#[hi::marker]
pub trait Parameter<Q: Qualifier<Storage>>: glsl::bounds::TransparentType { }


#[hi::marker]
trait ParameterDH<S: glsl::valid::Subtype, Q: Qualifier<Storage>>: Parameter<Q> + glsl::bounds::TransparentType<Subtype = S> { }

impl<T, Q> ParameterDH<glsl::valid::Scalar, Q> for T
where 
    T: glsl::bounds::ScalarType, Q: Qualifier<Storage> { }

impl<T, Q, const DIM: usize> ParameterDH<glsl::valid::Vector<DIM>, Q> for T
where 
    T: glsl::bounds::VectorType<DIM>, Q: Qualifier<Storage> , Const<DIM>: glsl::valid::VecDim{ }

impl<T, Q, S> Parameter<Q> for T
where
    S: glsl::valid::Subtype, T: glsl::bounds::TransparentType<Subtype = S> + ParameterDH<S, Q>, Q: Qualifier<Storage> { }


/// Marker trait for types that represent program / shader inputs and outputs.
#[hi::marker]
pub trait Parameters<Q>: HList
where
    Q: Qualifier<Storage>,
{ }

impl<Q> Parameters<Q> for () where Q: Qualifier<Storage> { }

impl<Head, T, const LOCATION: usize> Parameters<storage::In> for (Head, InBinding<T, LOCATION>)
where
    Head: Parameters<storage::In>,
    T: glsl::Type,
{ }

impl<Head, T, const LOCATION: usize> Parameters<storage::Out> for (Head, OutBinding<T, LOCATION>)
where
    Head: Parameters<storage::Out>,
    T: glsl::Type,
{ }


#[cfg(test)]
mod tests {
    use super::*;

    use crate::gl::buffer;

    fn require_parameter<P: Parameter<buffer::target::Array>>() { }

    #[test]
    fn are_scalars_valid() {
        require_parameter::<f32>();
        require_parameter::<f64>();
        require_parameter::<u32>();
        require_parameter::<i32>();
    }

    #[test]
    fn are_vectors_valid() {
        require_parameter::<glsl::Vec2>();
        require_parameter::<glsl::Vec3>();
        require_parameter::<glsl::Vec4>();
        
        require_parameter::<glsl::DVec2>();
 ``       require_parameter::<glsl::DVec3>();
        require_parameter::<glsl::DVec4>();
        
        require_parameter::<glsl::IVec2>();
        require_parameter::<glsl::IVec3>();
        require_parameter::<glsl::IVec4>();
        
        require_parameter::<glsl::UVec2>();
        require_parameter::<glsl::UVec3>();
        require_parameter::<glsl::UVec4>();
    }
}