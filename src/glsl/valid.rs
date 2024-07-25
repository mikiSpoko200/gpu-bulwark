use crate::prelude::internal::*;

use bounds::TransparentType;

use super::*;

/// Types that qualify family of glsl types.
#[hi::marker]
pub trait Subtype { }

/// Types that qualify group of glsl type.
#[hi::marker]
pub trait TypeGroup { }

/// Qualifier for scalar types in glsl.
#[hi::mark(Subtype)]
pub enum Scalar { }

/// Qualifier for vector types in glsl.
#[hi::mark(Subtype)]
pub enum Vector<const DIM: usize> { }

/// Qualifier for matrix types in glsl.
#[hi::mark(Subtype)]
pub enum Matrix { }

#[hi::mark(Subtype)]
pub struct Array<S>(PhantomData<S>)
where
    S: Subtype;

#[derive(Clone, Copy, Debug)]
#[hi::mark(TypeGroup)]
pub enum Transparent { }

#[derive(Clone, Copy, Debug)]
#[hi::mark(TypeGroup)]
pub enum Opaque { }

/// Types valid for use as glsl scalar.
#[hi::marker]
pub trait ForScalar: TransparentType + bounds::ScalarType { }

pub trait VecDim { }

hi::denmark! { Const<2> as VecDim }
hi::denmark! { Const<3> as VecDim }
hi::denmark! { Const<4> as VecDim }

/// Types valid for use as glsl vectors.
#[hi::marker]
pub trait ForVector<const DIM: usize>: TransparentType + Location<Vector<DIM>> + bounds::ScalarType
where 
    Const<DIM>: VecDim
{ }

/// Types valid for use as glsl matrices.
#[hi::marker]
pub trait ForMatrix<const ROW: usize, const COL: usize>: TransparentType + Location<Vector<COL>> + bounds::ScalarType
where
    Const<ROW>: valid::VecDim,
    Const<COL>: valid::VecDim,
{ }

/// Types valid for use as glsl arrays.
/// TODO: Check if arrays can indeed store arbitrary types?
#[hi::marker]
pub trait ForArray: super::Type { }

/// Types which are valid for use in 
impl<T> ForScalar for T where T: bounds::ScalarType { }

/// Any type valid for use as scalar is valid for Vector use.
impl<T, const DIM: usize> ForVector<DIM> for T
where 
    T: ForScalar + Location<Vector<DIM>>, 
    Const<DIM>: valid::VecDim
{ }

impl<const R: usize, const C: usize> ForMatrix<R, C> for f32
where
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{ }

impl<const R: usize, const C: usize> ForMatrix<R, C> for f64
where
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{ }

impl<T> ForArray for T where T: Type { }

// =================[ Opaque types ]================= //

pub trait ForSampler: Type { }

hi::denmark! { f32 as ForSampler }
hi::denmark! { i32 as ForSampler }
hi::denmark! { u32 as ForSampler }
