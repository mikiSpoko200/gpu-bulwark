//! This module provides `location` glsl attribute calculations for glsl types.

use crate::glsl::types;
use types::{Const, VecSize};
use crate::types::Unimplemented;

/// Trait that abstracts over GLSL location count that the type uses.
pub unsafe trait Layout {
    const LOCATION_COUNT: usize;
}

unsafe impl Layout for Unimplemented {
    const LOCATION_COUNT: usize = 0;
}

unsafe impl Layout for f32 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl Layout for f64 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl Layout for i32 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl Layout for u32 {
    const LOCATION_COUNT: usize = 1;
}

unsafe impl Layout for bool {
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl vecX types.
unsafe impl<const SIZE: usize> Layout for types::Vec<f32, SIZE>
where
    Const<SIZE>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl ivecX types.
unsafe impl<const SIZE: usize> Layout for types::IVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl uvecX types.
unsafe impl<const SIZE: usize> Layout for types::UVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl bvecX types.
unsafe impl<const SIZE: usize> Layout for types::BVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl dvecX types is different dvec2 take 1 location and dvec3/4 take 2.
unsafe impl<const SIZE: usize> Layout for types::DVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const LOCATION_COUNT: usize = match SIZE {
        2 => 1,
        3 | 4 => 2,
        _ => panic!("invalid Vec size"),
    };
}

unsafe impl<T, const N: usize> Layout for types::Array<T, N>
where
    T: Layout,
{
    const LOCATION_COUNT: usize = T::LOCATION_COUNT * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
unsafe impl<T, const ROW: usize, const COL: usize> Layout for types::Mat<T, ROW, COL>
where
    T: Layout,
    Const<ROW>: VecSize,
    Const<COL>: VecSize,
    types::Vec<T, COL>: Layout,
{
    const LOCATION_COUNT: usize = <types::Array<types::Vec<T, COL>, ROW> as Layout>::LOCATION_COUNT;
}
