#![allow(unused)]

//! Definitions of glsl types.

use super::location;
use std::marker::PhantomData;


/// Marker trait for glsl types.
pub unsafe trait Type: location::marker::Location + Default + Clone + Sized {}

/// Marker trait for glsl scalar types.
pub trait Scalar: Type {}

unsafe impl Type for f32 {}
impl Scalar for f32 {}

unsafe impl Type for f64 {}
impl Scalar for f64 {}

unsafe impl Type for i32 {}
impl Scalar for i32 {}

unsafe impl Type for u32 {}
impl Scalar for u32 {}

unsafe impl Type for bool {}
impl Scalar for bool {}

/// Wrapper for integer values that moves them into type system.
pub(crate) struct Const<const NUMBER: usize>;

pub trait VecSize {}

impl VecSize for Const<2> {}

impl VecSize for Const<3> {}

impl VecSize for Const<4> {}

pub mod base {
    use super::{VecSize, Const, PhantomData, Type};

    /// Generic basis for GLSL Vectors. 
    /// GLSL Vectors can contain multiple data types but can only appear in sized of 2, 3 or 4.
    /// This constraint is represented by trait bound `VecSize` on `Const`.
    #[derive(Clone, Debug, Default)]
    pub struct Vec<T, const SIZE: usize>(PhantomData<T>)
    where
        Const<SIZE>: VecSize,
        T: Type,
    ;
}

/// Vector of single precision floats.
pub type Vec<const N: usize> = base::Vec<f32, N>;

pub type Vec2 = Vec<2>;
pub type Vec3 = Vec<3>;
pub type Vec4 = Vec<4>;

unsafe impl<const N: usize> Type for Vec<N> where Const<N>: VecSize {}


/// Vector of signed integers.
pub type IVec<const N: usize> = base::Vec<i32, N>;

pub type IVec2 = IVec<2>;
pub type IVec3 = IVec<3>;
pub type IVec4 = IVec<4>;

/// Vector of integers is a valid type.
unsafe impl<const N: usize> Type for IVec<N> where Const<N>: VecSize {}

/// Vector of unsigned integers.
pub type UVec<const N: usize> = base::Vec<u32, N>;

pub type UVec2 = UVec<2>;
pub type UVec3 = UVec<3>;
pub type UVec4 = UVec<4>;

/// Vector of unsigned integers is a valid type.
unsafe impl<const N: usize> Type for UVec<N> where Const<N>: VecSize {}

/// Vector of Doubles.
pub type DVec<const N: usize> = base::Vec<f64, N>;

pub type DVec2 = DVec<2>;
pub type DVec3 = DVec<3>;
pub type DVec4 = DVec<4>;

/// Vector of double precision floats is a valid type.
unsafe impl<const N: usize> Type for DVec<N> where Const<N>: VecSize {}

/// SAFETY: note bool here may be ABI incompatible
pub type BVec<const N: usize> = base::Vec<bool, N>;

pub type BVec2 = BVec<2>;
pub type BVec3 = BVec<3>;
pub type BVec4 = BVec<4>;

/// Vector of booleans is a valid type.
unsafe impl<const N: usize> Type for BVec<N> where Const<N>: VecSize {}


/// Matrix is in fact just a Vector of Vectors.
///
/// Array is not used here since not all Array sizes are valid Matrices.
/// Vectors on the contrary fit here perfectly.
pub type Mat<T, const ROW: usize, const COL: usize = ROW> = base::Vec<base::Vec<T, COL>, ROW>;
/// Vector of single precision floats is a valid type.

pub type Mat2 = Mat<f32, 2, 2>;
pub type Mat2x2 = Mat<f32, 2, 2>;
pub type Mat2x3 = Mat<f32, 2, 3>;
pub type Mat2x4 = Mat<f32, 2, 4>;
pub type Mat3x2 = Mat<f32, 3, 2>;
pub type Mat3 = Mat<f32, 3>;
pub type Mat3x3 = Mat<f32, 3, 3>;
pub type Mat3x4 = Mat<f32, 3, 4>;
pub type Mat4x2 = Mat<f32, 4, 2>;
pub type Mat4x3 = Mat<f32, 4, 3>;
pub type Mat4 = Mat<f32, 4>;
pub type Mat4x4 = Mat<f32, 4, 4>;

/// Single precision matrix is a valid type.
unsafe impl<const R: usize, const C: usize> Type for Mat<f32, R, C>
where
    Const<R>: VecSize,
    Const<C>: VecSize,
{
}

pub type DMat2 = Mat<f64, 2>;
pub type DMat2x2 = Mat<f64, 2, 2>;
pub type DMat2x3 = Mat<f64, 2, 3>;
pub type DMat2x4 = Mat<f64, 2, 4>;
pub type DMat3x2 = Mat<f64, 3, 2>;
pub type DMat3 = Mat<f64, 3>;
pub type DMat3x3 = Mat<f64, 3, 3>;
pub type DMat3x4 = Mat<f64, 3, 4>;
pub type DMat4x2 = Mat<f64, 4, 2>;
pub type DMat4x3 = Mat<f64, 4, 3>;
pub type DMat4 = Mat<f64, 4>;
pub type DMat4x4 = Mat<f64, 4, 4>;

/// Double precision matrix is a valid type.
unsafe impl<const R: usize, const C: usize> Type for Mat<f64, R, C>
where
    Const<R>: VecSize,
    Const<C>: VecSize,
{
}

/// GLSL array.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Array<T, const N: usize>(PhantomData<T>) where T: Type;

/// Array of types is a valid type.
unsafe impl<T, const N: usize> Type for Array<T, N> where T: Type {}
