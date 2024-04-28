//! This module provides `location` glsl attribute calculations for glsl types.

use crate::{glsl::types, prelude::HList};
use types::{Const, VecSize};

use std::marker::PhantomData;

use super::{Uniform, Type};

pub mod marker {
    pub unsafe trait Location {
        const SIZE: usize;
    }
}

unsafe impl marker::Location for f32 {
    const SIZE: usize = 1;
}

unsafe impl marker::Location for f64 {
    const SIZE: usize = 1;
}

unsafe impl marker::Location for i32 {
    const SIZE: usize = 1;
}

unsafe impl marker::Location for u32 {
    const SIZE: usize = 1;
}

unsafe impl marker::Location for bool {
    const SIZE: usize = 1;
}

/// Location count for glsl vecX types.
unsafe impl<const SIZE: usize> marker::Location for types::Vec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const SIZE: usize = 1;
}

/// Location count for glsl ivecX types.
unsafe impl<const SIZE: usize> marker::Location for types::IVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const SIZE: usize = 1;
}

/// Location count for glsl uvecX types.
unsafe impl<const SIZE: usize> marker::Location for types::UVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const SIZE: usize = 1;
}

/// Location count for glsl dvecX types is different dvec2 take 1 location and dvec3/4 take 2.
unsafe impl<const SIZE: usize> marker::Location for types::DVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const SIZE: usize = match SIZE {
        2 => 1,
        3 | 4 => 2,
        _ => unreachable!(), // VecSize bound should prevent this code from being ever reached
    };
}

/// Location count for glsl bvecX types.
unsafe impl<const SIZE: usize> marker::Location for types::BVec<SIZE>
where
    Const<SIZE>: VecSize,
{
    const SIZE: usize = 1;
}

/// Location for an Array of `T` where `T: Location` of size `N` is `N * <T as Location>::LOCATION_COUNT`,
unsafe impl<T, const N: usize> marker::Location for types::Array<T, N>
where
    T: Type,
{
    const SIZE: usize = T::SIZE * N;
}

/// Accordingly to GLSL spec matrices use the same number of locations as arrays of Row
unsafe impl<T, const ROW: usize, const COL: usize> marker::Location for types::Mat<T, ROW, COL>
where
    T: Type,
    Const<ROW>: VecSize,
    Const<COL>: VecSize,
    types::base::Vec<T, COL>: Type,
{
    const SIZE: usize = <types::Array<types::base::Vec<T, COL>, ROW> as marker::Location>::SIZE;
}

trait ValidationStatus { }

#[derive(Clone, Copy, Debug)]
pub struct Unvalidated;
#[derive(Clone, Copy, Debug)]
pub struct Validated;

impl ValidationStatus for Unvalidated {} 
impl ValidationStatus for Validated {} 

#[derive(Clone, Copy, Debug)]
pub struct Location<U, const LOCATION: usize, V>(PhantomData<U>, PhantomData<V>) where U: Uniform;

impl<U, const LOCATION: usize> Default for Location<U, LOCATION, Unvalidated>
where
    U: Uniform
{
    fn default() -> Self {
        Self::new()
    }
}

impl<U, const LOCATION: usize> Location<U, LOCATION, Unvalidated>
where
    U: Uniform
{
    const fn new() -> Self {
        Self(PhantomData, PhantomData)
    }
    
    fn validate(self) -> Location<U, LOCATION, Validated> {
        Location(PhantomData, PhantomData)
    }
}

const fn validate_location<PU, const PREV_LOCATION: usize, CU, const CURR_LOCATION: usize>() -> usize
where
    PU: Uniform,
    CU: Uniform,
{
    assert!(!(PREV_LOCATION > CURR_LOCATION + CU::SIZE), "locations must be specified in strictly increasing order");
    assert!(PREV_LOCATION + PU::SIZE <= CURR_LOCATION, "locations overlap");
    CURR_LOCATION
}

pub trait Locations<ValidationStatus>: HList {
    type Uniform: Uniform;
    const LOCATION: usize;
    type Validated: HList;
    
    fn validate(self) -> Self::Validated;
}

impl<U, const LOCATION: usize> Locations<Unvalidated> for ((), Location<U, LOCATION, Unvalidated>)
where
    U: Uniform
{
    type Uniform = U;
    const LOCATION: usize = LOCATION;
    type Validated = ((), Location<U, LOCATION, Validated>);
    
    fn validate(self) -> Self::Validated {
        let _: usize = Self::LOCATION;
        ((), self.1.validate())
    }
}

impl<U, const LOCATION: usize> Locations<Validated> for ((), Location<U, LOCATION, Validated>)
where
    U: Uniform
{
    type Uniform = U;
    const LOCATION: usize = LOCATION;
    type Validated = Self;
    
    fn validate(self) -> Self::Validated {
        self
    }
}

impl<H, PU, CU, const PREV_LOCATION: usize, const CURR_LOCATION: usize> Locations<Unvalidated> for ((H, Location<PU, PREV_LOCATION, Validated>), Location<CU, CURR_LOCATION, Unvalidated>)
where
    (H, Location<PU, PREV_LOCATION, Validated>): Locations<Validated>,
    H: HList,
    PU: Uniform,
    CU: Uniform
{
    type Uniform = CU;
    const LOCATION: usize = validate_location::<PU, PREV_LOCATION, CU, CURR_LOCATION>();
    type Validated = (<(H, Location<PU, PREV_LOCATION, Validated>) as Locations<Validated>>::Validated, Location<CU, CURR_LOCATION, Validated>);
    
    fn validate(self) -> Self::Validated {
        let _: usize = Self::LOCATION;
        let (head, location) = self;
        (head.validate(), location.validate())
    }
}

impl<H, PU, CU, const PREV_LOCATION: usize, const CURR_LOCATION: usize> Locations<Validated> for ((H, Location<PU, PREV_LOCATION, Validated>), Location<CU, CURR_LOCATION, Validated>)
where
    (H, Location<PU, PREV_LOCATION, Validated>): Locations<Validated>,
    H: HList,
    PU: Uniform,
    CU: Uniform
{
    type Uniform = CU;
    const LOCATION: usize = CURR_LOCATION;
    type Validated = Self;
    
    fn validate(self) -> Self::Validated {
        self
    }
}

// TODO: Work on dsl syntax
#[macro_export]
macro_rules! locations {
    (Location<$type: ty, $location: literal> $(,)?) => {
        ((), Location::<$type, $location, _>::default()).validate()
    };
    (Location<$type: ty, $location: literal>, $(Location<$types: ty, $locations: literal>),* $(,)?) => {
        locations!(@ ((), crate::glsl::location::Location::<$type, $location, _>::default()).validate(), $(Location<$types, $locations>),*)
    };
    (@ $acc: expr, Location<$type: ty, $location: literal> $(,)?) => {
        ($acc, crate::glsl::location::Location::<$type, $location, _>::default()).validate()
    };
    (@ $acc: expr, Location<$type: ty, $location: literal>, $(Location<$types: ty, $locations: literal>),*) => {
        locations!(@ ($acc, crate::glsl::location::Location::<$type, $location, _>::default()).validate(), $(Location<$types, $locations>),*)
    };
}
