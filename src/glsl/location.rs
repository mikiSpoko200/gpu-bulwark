//! This module provides `location` glsl attribute calculations for glsl types.

use crate::glsl;
use crate::prelude::internal::*;
use crate::valid;


pub trait Location<Subtype: valid::Subtype = valid::Scalar> {
    const N_USED_LOCATIONS: usize;
}


macro_rules! impl_location {
    ([scalar] $ty:ty: $location:expr) => {
        impl Location for $ty {
            const N_USED_LOCATIONS: usize = $location;
        }
    };
    ([vector] $ty:ty: $location:expr) => {
        impl<const DIM: usize> Location<valid::Vector<DIM>> for $ty {
            const N_USED_LOCATIONS: usize = $location;
        }
    }
}

impl_location!{ [scalar] f32: 1 }
impl_location!{ [scalar] f64: 1 }
impl_location!{ [scalar] i32: 1 }
impl_location!{ [scalar] u32: 1 }

impl_location!{ [vector] f32: 1 }
impl_location!{ [vector] f64: match DIM { 2 => 1, 3 | 4 => 2, _ => panic!("unreachable") } }
impl_location!{ [vector] i32: 1 }
impl_location!{ [vector] u32: 1 }

/// Implementation of vector location count is delegated to type `T` via `valid::ForVector`'s supertrait `Location<valid::Vector<DIM>>`.
impl<T, const DIM: usize> Location for glsl::GVec<T, DIM>
where
    T: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim,
{
    const N_USED_LOCATIONS: usize = <T as Location<valid::Vector<DIM>>>::N_USED_LOCATIONS;
}

/// Location for an Array of `T` where `T: Location` of size `N` is `N * <T as Location>::LOCATION_COUNT`,
impl<T, const N: usize> Location for glsl::Array<T, N>
where
    T: glsl::Type + Location,
{
    const N_USED_LOCATIONS: usize = T::N_USED_LOCATIONS * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
impl<T, const R: usize, const C: usize> Location for glsl::Mat<T, R, C>
where
    T: valid::ForMatrix<R, C>,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    const N_USED_LOCATIONS: usize = <glsl::Array<glsl::GVec<T, C>, R> as Location>::N_USED_LOCATIONS;
}
