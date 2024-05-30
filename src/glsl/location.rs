//! This module provides `location` glsl attribute calculations for glsl types.

use crate::glsl;
use glsl::{Const, Type};

use self::marker::Location;

use super::marker::{MatrixType, ScalarType};

pub mod marker {
    pub unsafe trait Location {
        const LOCATION_COUNT: usize;
    }
}

unsafe impl marker::Location for f32 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl marker::Location for f64 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl marker::Location for i32 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl marker::Location for u32 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl marker::Location for bool {
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl vecX types.
unsafe impl<const VEC_SIZE: usize> marker::Location for glsl::Vec<VEC_SIZE>
where
    Const<VEC_SIZE>: glsl::marker::VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl ivecX types.
unsafe impl<const VEC_SIZE: usize> marker::Location for glsl::IVec<VEC_SIZE>
where
    Const<VEC_SIZE>: glsl::marker::VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl uvecX types.
unsafe impl<const VEC_SIZE: usize> marker::Location for glsl::UVec<VEC_SIZE>
where
    Const<VEC_SIZE>: glsl::marker::VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl dvecX types is different dvec2 take 1 location and dvec3/4 take 2.
unsafe impl<const VEC_SIZE: usize> marker::Location for glsl::DVec<VEC_SIZE>
where
    Const<VEC_SIZE>: glsl::marker::VecSize,
{
    const LOCATION_COUNT: usize = match VEC_SIZE {
        2 => 1,
        3 | 4 => 2,
        _ => unreachable!(), // VecSize bound should prevent this code from being ever reached
    };
}

/// Location count for glsl bvecX types.
// unsafe impl<const VEC_SIZE: usize> marker::Location for glsl::BVec<VEC_SIZE>
// where
//     Const<VEC_SIZE>: glsl::marker::VecSize,
// {
//     const LOCATION_COUNT: usize = 1;
// }

/// Location for an Array of `T` where `T: Location` of size `N` is `N * <T as Location>::LOCATION_COUNT`,
unsafe impl<T, const N: usize> marker::Location for glsl::Array<T, N>
where
    T: Type,
{
    const LOCATION_COUNT: usize = T::LOCATION_COUNT * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
unsafe impl<T, const ROW_SIZE: usize, const COL_SIZE: usize> marker::Location
    for glsl::Mat<T, ROW_SIZE, COL_SIZE>
where
    T: ScalarType,
    Const<ROW_SIZE>: glsl::marker::VecSize,
    Const<COL_SIZE>: glsl::marker::VecSize,
    glsl::Mat<T, ROW_SIZE, COL_SIZE>: MatrixType,
    glsl::base::Vec<T, COL_SIZE>: Location,
{
    const LOCATION_COUNT: usize =
        <glsl::Array<glsl::base::Vec<T, COL_SIZE>, ROW_SIZE> as marker::Location>::LOCATION_COUNT;
}
