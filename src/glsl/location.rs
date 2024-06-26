//! This module provides `location` glsl attribute calculations for glsl types.

use crate::glsl;
use crate::prelude::internal::*;
use crate::valid;

/// If anyone thinks this should be unsafe come and fight me!
#[sealed]
pub trait Location {
    const LOCATION_COUNT: usize;
}

macro_rules! impl_location {
    ($ty: ty: $location: literal) => {
        #[sealed]
        impl Location for $ty {
            const LOCATION_COUNT: usize = $location;
        }
    }
}

impl_location!{ f32: 1 }
impl_location!{ f64: 1 }
impl_location!{ i32: 1 }
impl_location!{ u32: 1 }

/// Location count for glsl vecX types.
impl<const VEC_SIZE: usize> Location for glsl::Vec<VEC_SIZE>
where
    Const<VEC_SIZE>: valid::ForVector,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl ivecX types.
impl<const VEC_SIZE: usize> Location for glsl::IVec<VEC_SIZE>
where
    Const<VEC_SIZE>: valid::ForVector,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl uvecX types.
impl<const VEC_SIZE: usize> Location for glsl::UVec<VEC_SIZE>
where
    Const<VEC_SIZE>: valid::ForVector,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl dvecX types is different dvec2 take 1 location and dvec3/4 take 2.
impl<const VEC_SIZE: usize> Location for glsl::DVec<VEC_SIZE>
where
    Const<VEC_SIZE>: valid::ForVector,
{
    const LOCATION_COUNT: usize = match VEC_SIZE {
        2 => 1,
        3 | 4 => 2,
        _ => unreachable!(""), // `Const<VEC_SIZE>: valid::ForVector` should prevent this code from being ever reached
    };
}

/// Location for an Array of `T` where `T: Location` of size `N` is `N * <T as Location>::LOCATION_COUNT`,
impl<T, const N: usize> Location for glsl::Array<T, N>
where
    T: glsl::Type,
{
    const LOCATION_COUNT: usize = T::LOCATION_COUNT * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
impl<T, const ROW_SIZE: usize, const COL_SIZE: usize> Location for glsl::Mat<T, ROW_SIZE, COL_SIZE>
where
    T: glsl::Type + valid::ForMatrix,
    Const<ROW_SIZE>: valid::ForVector,
    Const<COL_SIZE>: valid::ForVector,
{
    const LOCATION_COUNT: usize = <glsl::Array<glsl::GVec<T, COL_SIZE>, ROW_SIZE> as Location>::LOCATION_COUNT;
}
