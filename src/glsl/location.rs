//! This module provides `location` glsl attribute calculations for glsl types.

use crate::glsl;
use crate::prelude::internal::*;
use crate::valid;


trait LocationHelper<const N: usize> {
    const LOCATION_COUNT: usize;
}

impl<const N: usize> LocationHelper<N> for glsl::DVec<N>
where
    Const<N>: valid::VecDim
{
    const LOCATION_COUNT: usize = match N { 2 => 1, 3 | 4 => 2, _ => unreachable!("") };
}

macro_rules! impl_location {
    ([scalar] $ty:ty: $location:expr, $subtype:path) => {
        impl LocationHelper<1> for $ty {
            const LOCATION_COUNT: usize = $location;
        }
    };
    ([vector] $ty:ty: $location:expr, $subtype:path) => {
        impl<const N: usize> LocationHelper<N> for glsl::GVec<$ty, N>
        where
            Const<N>: valid::VecDim
        {
            const LOCATION_COUNT: usize = $location;
        }
    };
    ([matrix] $ty:ty: $location:expr, $subtype:path) => {
        impl<const N: usize> LocationHelper<N> for glsl::GVec<$ty, N>
        where
            Const<N>: valid::VecDim
        {
            const LOCATION_COUNT: usize = $location;
        }
    }
}

impl_location!{ [scalar] f32: 1, valid::Scalar }
impl_location!{ [scalar] f64: 1, valid::Scalar }
impl_location!{ [scalar] i32: 1, valid::Scalar }
impl_location!{ [scalar] u32: 1, valid::Scalar }

impl_location!{ [vector] f32: 1, valid::Vector }
impl_location!{ [vector] f64: match N { 2 => 1, 3 | 4 => 2, _ => unreachable!("") }, valid::Vector }
impl_location!{ [vector] i32: 1, valid::Vector }
impl_location!{ [vector] u32: 1, valid::Vector }


/// If anyone thinks this should be unsafe come and fight me!
pub trait Location where {
    const LOCATION_COUNT: usize;
}

/// Location for an Array of `T` where `T: Location` of size `N` is `N * <T as Location>::LOCATION_COUNT`,
impl<T, const N: usize> Location for glsl::Array<T, N>
where
    T: glsl::Type,
{
    const LOCATION_COUNT: usize = T::LOCATION_COUNT * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
impl<T, const ROW_SIZE: usize, const COL_SIZE: usize> Location<valid::Matrix> for glsl::Mat<T, ROW_SIZE, COL_SIZE>
where
    T: glsl::Type + valid::ForMatrix,
    Const<ROW_SIZE>: valid::VecDim,
    Const<COL_SIZE>: valid::VecDim,
{
    const LOCATION_COUNT: usize = <glsl::Array<glsl::GVec<T, COL_SIZE>, ROW_SIZE> as LocationBlanketHelper>::LOCATION_COUNT;
}
