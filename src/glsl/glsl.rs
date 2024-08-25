#![allow(unused)]

use crate::disjoint;
use crate::gl;
use crate::prelude::internal::*;

use crate::ext;
use crate::ffi;

use super::location::Location;
use crate::prelude::*;

use glsl::sampler;
use glsl::valid;
use gl::texture;

pub use glsl::sampler::*;

/// A glsl type.
pub trait Type {
    type Group: valid::TypeGroup;
}

/// Common trait bound combinations.
pub mod bounds {
    use super::*;

    use crate::ffi;
    use crate::prelude::internal::*;

    pub trait TransparentType: Type<Group=valid::Transparent> + Location + Default + Clone + Sized + ffi::FFI {
        type Subtype: valid::Subtype;
    }

    /// TODO: Do opaque types use locations?
    #[hi::marker]
    pub trait OpaqueType: Type<Group=valid::Opaque> { }

    #[hi::marker]
    pub trait ScalarType: TransparentType<Subtype=valid::Scalar> { }

    #[hi::marker]
    pub trait VectorType<const DIM: usize>: TransparentType<Subtype=valid::Vector<DIM>>
    where
        Const<DIM>: valid::VecDim
    { }
    
    #[hi::marker]
    pub trait MatrixType: TransparentType<Subtype=valid::Matrix> { }
}

/// Traits for validation markers.

// ================[ Types ]================ //

/// Generic basis for GLSL Vector types.
/// 
/// GLSL Vectors can contain only specific data types and can only appear in sizes of 2, 3 or 4.
/// Requirements for generic parameters, both type param and const param, are expressed using
/// `valid::ForVector` (Bound on `Const<N>` in case of const param).
#[derive(Clone, Debug, Default)]
pub struct GVec<T, const DIM: usize>(PhantomData<T>)
where
    T: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim,
;

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
    T: valid::ForMatrix<ROW, COL>,
    Const<ROW>: valid::VecDim,
    Const<COL>: valid::VecDim,
;

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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Array<T, const N: usize>(PhantomData<T>)
where
    T: Type;

// =================[ impl Type / TransparentType ]================= //

macro_rules! impl_transparent {
    ($ty: ty as $subtype:path) => {
        impl crate::glsl::Type for $ty {
            type Group = crate::glsl::valid::Transparent;
        }
        impl crate::glsl::bounds::TransparentType for $ty {
            type Subtype = $subtype;
        }
    }
}

impl_transparent! { f32 as valid::Scalar }
impl_transparent! { f64 as valid::Scalar }
impl_transparent! { i32 as valid::Scalar }
impl_transparent! { u32 as valid::Scalar }

hi::denmark! { f32 as bounds::ScalarType }
hi::denmark! { f64 as bounds::ScalarType }
hi::denmark! { i32 as bounds::ScalarType }
hi::denmark! { u32 as bounds::ScalarType }

// `Type` impls for Vectors.

impl<T, const DIM: usize> Type for GVec<T, DIM>
where
    T: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim,
{
    type Group = valid::Transparent;
}

impl<T, const DIM: usize> bounds::TransparentType for GVec<T, DIM>
where 
    T: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim,
{
    type Subtype = valid::Vector<DIM>;
}

impl<T, const DIM: usize> bounds::VectorType<DIM> for GVec<T, DIM>
where
    T: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim
{ }

// `Type` impls for Matrices.

impl<T, const R: usize, const C: usize> Type for Mat<T, R, C>
where
    T: valid::ForMatrix<R, C>,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    type Group = valid::Transparent;
}

impl<T, const R: usize, const C: usize> bounds::TransparentType for Mat<T, R, C>
where
    T: valid::ForMatrix<R, C>,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    type Subtype = valid::Matrix;
}

impl<T, const R: usize, const C: usize> bounds::MatrixType for Mat<T, R, C>
where 
    T: valid::ForMatrix<R, C>,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{ }

// `Type` impls for Array.

impl<T, const N: usize> Type for Array<T, N>
where
    T: Type,
{
    type Group = T::Group;
}

impl<T, const N: usize> bounds::TransparentType for Array<T, N>
where
    T: bounds::TransparentType
{
    type Subtype = valid::Array<T::Subtype>;
}

// =================[ impl FFI ]================= //

unsafe impl<T, const DIM: usize> ffi::FFI for GVec<T, DIM>
where
    T: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim,
{
    type Layout = [T::Layout; DIM];
}

unsafe impl<T, const R: usize, const C: usize> ffi::FFI for Mat<T, R, C>
where
    T: valid::ForMatrix<R, C>,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    type Layout = [[T::Layout; C]; R];
}

unsafe impl<T, const N: usize> ffi::FFI for Array<T, N>
where
    T: bounds::TransparentType,
{
    type Layout = [T::Layout; N];
}

// =================[ impl Disjoint ]================= //

impl<T> Disjoint for T where T: TransparentType {
    type Discriminant = T::Subtype;
}

// =================[ Opaque types ]================= //

use bounds::TransparentType;

impl<T, O> Type for GSampler<T, O>
where
    T: texture::Target,
    O: sampler::Output,
{
    type Group = valid::Opaque;
}

impl<T, O> bounds::OpaqueType for GSampler<T, O>
where
    T: texture::Target,
    O: sampler::Output,
{ }
