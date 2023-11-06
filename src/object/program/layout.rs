//! This module provides `location` glsl attribute calculations for glsl types.

use crate::glsl::types::*;
use std::marker::PhantomData;

/// Trait that abstracts over GLSL location count that the type uses.
pub unsafe trait Layout {
    const LOCATION_COUNT: usize;
}

/// Location count for glsl vecX types.
unsafe impl<const Size: usize> Layout for Vec<f32, Size>
where
    Const<Size>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl ivecX types.
unsafe impl<const Size: usize> Layout for IVec<Size>
where
    Const<Size>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl uvecX types.
unsafe impl<const Size: usize> Layout for UVec<Size>
where
    Const<Size>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl bvecX types.
unsafe impl<const Size: usize> Layout for BVec<Size>
where
    Const<Size>: VecSize,
{
    const LOCATION_COUNT: usize = 1;
}

/// Location count for glsl dvecX types is different dvec2 take 1 location and dvec3/4 take 2.
unsafe impl Layout for DVec<2> {
    const LOCATION_COUNT: usize = 1;
}

/// SAFETY: double check is DVec3/4 for VS actually uses up 2 locations.
unsafe impl Layout for DVec<3> {
    const LOCATION_COUNT: usize = 2;
}

/// SAFETY: double check is DVec3/4 for VS actually uses up 2 locations.
unsafe impl Layout for DVec<4> {
    const LOCATION_COUNT: usize = 2;
}

/// Layout for an Array of `T` where `T: Layout` of size `N` is `N * <T as Layout>::LOCATION_COUNT`,
pub struct Array<T, const N: usize>(PhantomData<T>)
where
    T: Layout;

unsafe impl<T, const N: usize> Layout for Array<T, N>
where
    T: Layout,
{
    const LOCATION_COUNT: usize = T::LOCATION_COUNT * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
unsafe impl<T, const Row: usize, const Col: usize> Layout for Mat<T, Row, Col>
where
    T: Layout,
    Const<Row>: VecSize,
    Const<Col>: VecSize,
{
    const LOCATION_COUNT: usize = <Array<Vec<T, Col>, Row> as Layout>::LOCATION_COUNT;
}
