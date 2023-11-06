//! Definitions of glsl types.

use std::char::from_u32;
use std::marker::PhantomData;

/// Wrapper for integer values that moves them into type system.
pub struct Const<const Number: usize>;

pub trait VecSize {}

impl VecSize for Const<2> {}

impl VecSize for Const<3> {}

impl VecSize for Const<4> {}



fn foo() -> Vec<f32, 1> {

}

/// GLSL Vectors can contain multiple data types but can only appear in sized of 2, 3 or 4.
/// This constraint is represented by trait bound `VecSize` on `Const`.
pub struct Vec<T, const Size: usize>(PhantomData<T>)
where
    Const<Size>: VecSize;

/// Matrix is in fact just a Vector of Vectors.
///
/// Array is not used here since not all Array sizes are valid Matrices.
/// Vectors on the contrary fit here perfectly.
pub type Mat<T, const Row: usize, const Col: usize = Row>
where
    Const<Row>: VecSize,
    Const<Col>: VecSize,
= Vec<Vec<T, Col>, Row>;

pub type Vec2 = Vec<f32, 2>;
pub type Vec3 = Vec<f32, 3>;
pub type Vec4 = Vec<f32, 4>;

pub type IVec<const N: usize>
where
    Const<N>: VecSize,
= Vec<i32, N>;

pub type IVec2 = IVec<2>;
pub type IVec3 = IVec<3>;
pub type IVec4 = IVec<4>;

pub type UVec<const N: usize>
where
    Const<N>: VecSize,
= Vec<u32, N>;

pub type UVec2 = UVec<2>;
pub type UVec3 = UVec<3>;
pub type UVec4 = UVec<4>;

pub type DVec<const N: usize>
where
    Const<N>: VecSize,
= Vec<f64, N>;

pub type DVec2 = DVec<2>;
pub type DVec3 = DVec<3>;
pub type DVec4 = DVec<4>;

/// SAFETY: note bool here may be ABI incompatible
pub type BVec<const N: usize>
where
    Const<N>: VecSize,
= Vec<bool, N>;

pub type BVec2 = BVec<2>;
pub type BVec3 = BVec<3>;
pub type BVec4 = BVec<4>;

pub type Mat2 = Vec<Vec<f32, 2>, 2>;
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

/// Layout for an Array of `T` where `T: Layout` of size `N` is `N * <T as Layout>::LOCATION_COUNT`,
pub struct Array<T, const N: usize>(PhantomData<T>);
