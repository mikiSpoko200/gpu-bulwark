#![allow(unused)]

use std::marker::PhantomData;

use crate::{ext, ffi};

use super::location::Location;

/// Wrapper for integer values that moves them into type system.
/// Same trick is used in std here `https://doc.rust-lang.org/std/simd/prelude/struct.Simd.html`
pub(crate) struct Const<const NUMBER: usize>;


/// A glsl type.
pub trait Type {
    type Group: valid::TypeGroup;
}

/// Aliases for common trait bounds.
pub mod alias {
    use super::valid;
    use super::valid::Subtype;
    use super::Type;
    use crate::ffi;
    use super::Location;

    pub trait TransparentType: Type<Group=valid::Transparent> + Location + Default + Clone + Sized + ffi::FFI {
        type Subtype: valid::Subtype;
    }

    /// TODO: Do opaque types use locations?
    #[hi::marker]
    pub trait OpaqueType: Type<Group=valid::Opaque> { }
    
    #[hi::marker]
    pub trait ScalarType: TransparentType<Subtype=valid::Scalar> { }

    #[hi::marker]
    pub trait VectorType: TransparentType<Subtype=valid::Vector> { }
    
    #[hi::marker]
    pub trait MatrixType: TransparentType<Subtype=valid::Matrix> { }
}

/// Traits for validation markers.
pub mod valid {
    use hi;
    use super::*;

    /// Types that qualify familiy of glsl types.
    #[hi::marker]
    pub trait Subtype: crate::utils::Sealed { }
    
    /// Types that qualify group of glsl type.
    #[hi::marker]
    pub trait TypeGroup: crate::utils::Sealed { }
    
    /// Qualifier for scalar types in glgl.
    #[hi::mark(Subtype)]
    pub enum Scalar { }
    
    /// Qualifier for vector types in glgl.
    #[hi::mark(Subtype)]
    pub enum Vector { }
    
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
    pub trait ForScalar { }

    /// Types valid for use as glsl vectors.
    #[hi::marker]
    pub trait ForVector { }

    /// Types valid for use as glsl matrices.
    #[hi::marker]
    pub trait ForMatrix { }
    
    /// Types valid for use as glsl arrays.
    /// TODO: Check if arrays can indeed store arbitrary types?
    #[hi::marker]
    pub trait ForArray: super::Type { }

    impl ForVector for Const<2> { }
    impl ForVector for Const<3> { }
    impl ForVector for Const<4> { }
    
    /// Types which are valid for use in 
    impl<T> ForScalar for T where T: alias::ScalarType { }
    
    /// Any type valid for use as scalar is valid for Vector use.
    impl<T> ForVector for T where T: ForScalar { }
    
    impl ForMatrix for f32 { }
    impl ForMatrix for f64 { }

    impl<T, const N: usize> ForArray for T where T: Type { }
}

impl<T, const N: usize> Type for super::Array<T, N>
where
    T: Type,
{
    type Group = T::Group;
}

// ================[ Types ]================ //

/// Generic basis for GLSL Vector types.
/// 
/// GLSL Vectors can contain only specific data types and can only appear in sizes of 2, 3 or 4.
/// Requirements for generic parameters, both type param and const param, are expressed using
/// `valid::ForVector` (Bound on `Const<N>` in case of const param).
#[derive(Clone, Debug, Default)]
pub struct GVec<T, const SIZE: usize>(PhantomData<T>)
where
    Self: Location,
    Const<SIZE>: valid::ForVector,
    T: valid::ForVector;

/// Vector of single precision floats.
pub type Vec<const N: usize> = GVec<f32, N>;

pub type Vec2 = Vec<2>;
pub type Vec3 = Vec<3>;
pub type Vec4 = Vec<4>;

/// Vector of signed integers.
pub type IVec<const N: usize> = GVec<i32, N>;

pub type IVec2 = IVec<2>;
pub type IVec3 = IVec<3>;
pub type IVec4 = IVec<4>;

/// Vector of unsigned integers.
pub type UVec<const N: usize> = GVec<u32, N>;

pub type UVec2 = UVec<2>;
pub type UVec3 = UVec<3>;
pub type UVec4 = UVec<4>;
/// Vector of Doubles.
pub type DVec<const N: usize> = GVec<f64, N>;

pub type DVec2 = DVec<2>;
pub type DVec3 = DVec<3>;
pub type DVec4 = DVec<4>;

/// SAFETY: note bool here may be ABI incompatible
pub type BVec<const N: usize> = GVec<bool, N>;

pub type BVec2 = BVec<2>;
pub type BVec3 = BVec<3>;
pub type BVec4 = BVec<4>;

/// Matrix is in fact just a Vector of Vectors.
///
/// Array is not used here since not all Array sizes are valid Matrices.
/// Vectors on the contrary fit here perfectly.
#[derive(Clone, Debug, Default)]
pub struct Mat<T, const ROW: usize, const COL: usize = ROW>(PhantomData<T>)
where
    Const<ROW>: valid::ForVector,
    Const<COL>: valid::ForVector,
    T: valid::ForMatrix;

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

/// GLSL array.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Array<T, const N: usize>(PhantomData<T>)
where
    T: Type;

macro_rules! impl_transparent {
    ($ty: ty as $subtype: ident) => {
        impl crate::glsl::Type for $ty {
            type Group = crate::glsl::valid::Transparent;
        }
        impl crate::glsl::alias::TransparentType for $ty {
            type Subtype = crate::glsl::valid::$subtype;
        }
    }
}

impl_transparent! { f32 as Scalar }
impl_transparent! { f64 as Scalar }
impl_transparent! { i32 as Scalar }
impl_transparent! { u32 as Scalar }

// `Type` impls for Vectors.

impl<T, const N: usize> Type for GVec<T, N>
where
    T: valid::ForVector,
    Const<N>: valid::ForVector,
{
    type Group = valid::Transparent;
}

impl<T, const N: usize> alias::TransparentType for GVec<T, N>
where 
    Self: Location,
    T: valid::ForVector,
    Const<N>: valid::ForVector,
{
    type Subtype = valid::Vector;
}

// `Type` impls for Matrices.

impl<T, const R: usize, const C: usize> Type for Mat<T, R, C>
where
    T: valid::ForMatrix,
    Const<R>: valid::ForVector,
    Const<C>: valid::ForVector,
{
    type Group = valid::Transparent;
}

impl<T, const R: usize, const C: usize> alias::TransparentType for Mat<T, R, C>
where
    T: valid::ForMatrix,
    Const<R>: valid::ForVector,
    Const<C>: valid::ForVector,
{
    type Subtype = valid::Matrix;
}

// =================[ impl FFI ]================= //

unsafe impl<T, const N: usize> ffi::FFI for GVec<T, N>
where
    T: valid::ForVector + ext::Array,
    Const<N>: valid::ForVector,
{
    type Layout = [T; N];
}

unsafe impl<T, const R: usize, const C: usize> ffi::FFI for Mat<T, R, C>
where
    T: valid::ForMatrix,
    Const<R>: valid::ForVector,
    Const<C>: valid::ForVector,
{
    type Layout = [[T; C]; R];
}

unsafe impl<T, const N: usize> ffi::FFI for Array<T, N>
where
    T: alias::TransparentType,
{
    type Layout = [T::Layout; N];
}