//! This module provides specialization of HLists for Program / Shader parameters.

use crate::prelude::internal::*;
use crate::glsl;
use crate::hlist::lhlist::Base as HList;
use glsl::{InBinding, OutBinding, Storage, Qualifier, storage};


#[hi::marker]
pub trait Parameter<Q: Qualifier<Storage>>: glsl::bounds::TransparentType { }

#[hi::marker]
pub trait ValidForParameter<Q: Qualifier<Storage>> { }

impl<Q: Qualifier<Storage>> ValidForParameter<Q> for glsl::valid::Scalar { }

impl<const DIM: usize, Q> ValidForParameter<Q> for glsl::valid::Vector<DIM>
where
    Const<DIM>: glsl::valid::VecDim,
    Q: Qualifier<Storage>
{ }


// #[hi::marker]
// trait ParameterDH<S: glsl::valid::Subtype, Q: Qualifier<Storage>>: Parameter<Q> + glsl::bounds::TransparentType<Subtype = S> { }

// impl<T, Q> ParameterDH<glsl::valid::Scalar, Q> for T
// where 
//     T: glsl::bounds::ScalarType, Q: Qualifier<Storage> { }

// impl<T, Q, const DIM: usize> ParameterDH<glsl::valid::Vector<DIM>, Q> for T
// where 
//     T: glsl::bounds::VectorType<DIM>, Q: Qualifier<Storage> , Const<DIM>: glsl::valid::VecDim{ }

impl<T, Q> Parameter<Q> for T
where
    T: glsl::bounds::TransparentType<Subtype: ValidForParameter<Q>>, Q: Qualifier<Storage> { }


/// Marker trait for types that represent program / shader inputs and outputs.
#[hi::marker]
pub trait Parameters<Q>: HList
where
    Q: Qualifier<Storage>,
{}

impl<Q> Parameters<Q> for () where Q: Qualifier<Storage> { }

impl<PH, T, const LOCATION: usize> Parameters<storage::In> for (PH, InBinding<T, LOCATION>)
where
    PH: Parameters<storage::In>,
    T: Parameter<storage::In>,
{ }

impl<Head, T, const LOCATION: usize> Parameters<storage::Out> for (Head, OutBinding<T, LOCATION>)
where
    Head: Parameters<storage::Out>,
    T: Parameter<storage::Out>,
{ }


#[cfg(test)]
mod tests {
    use super::*;

    use glsl::binding::{Qualifier, Storage};

    fn require_parameter<P: Parameter<Q>, Q: Qualifier<Storage>>() { }

    fn are_scalars_valid<Q: Qualifier<Storage>>() {
        require_parameter::<f32, Q>();
        require_parameter::<f64, Q>();
        require_parameter::<u32, Q>();
        require_parameter::<i32, Q>();
    }

    #[test]
    fn are_scalars_valid_in_parameters() {
        are_scalars_valid::<storage::In>();
    }
    
    #[test]
    fn are_scalars_valid_out_parameters() {
        are_scalars_valid::<storage::Out>();
    }
    
    #[test]
    fn are_scalars_valid_uniform_parameters() {
        are_scalars_valid::<storage::Uniform>();
    }
    
    fn are_vectors_valid<Q: Qualifier<Storage>>() {
        require_parameter::<glsl::Vec2, Q>();
        require_parameter::<glsl::Vec3, Q>();
        require_parameter::<glsl::Vec4, Q>();
        
        require_parameter::<glsl::DVec2, Q>();
        require_parameter::<glsl::DVec3, Q>();
        require_parameter::<glsl::DVec4, Q>();
        
        require_parameter::<glsl::IVec2, Q>();
        require_parameter::<glsl::IVec3, Q>();
        require_parameter::<glsl::IVec4, Q>();
        
        require_parameter::<glsl::UVec2, Q>();
        require_parameter::<glsl::UVec3, Q>();
        require_parameter::<glsl::UVec4, Q>();
    }

    #[test]
    fn are_vectors_valid_in_parameters() {
        are_vectors_valid::<storage::In>();
    }

    #[test]
    fn are_vectors_valid_out_parameters() {
        are_vectors_valid::<storage::Out>();
    }

    #[test]
    fn are_vectors_valid_uniform_parameters() {
        are_vectors_valid::<storage::Uniform>();
    }
}