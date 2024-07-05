#![allow(unused)]

use crate::gl;
use crate::prelude::internal::*;

use crate::ext;
use crate::ffi;

use super::location::Location;
use crate::prelude::*;
use crate::valid;

/// A glsl type.
pub trait Type {
    type Group: valid::TypeGroup;
}

/// Common trait bound combinations.
pub mod bounds {
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
pub mod _valid {
    use bounds::{OpaqueType, TransparentType};

    use super::*;

    /// Types that qualify familiy of glsl types.
    #[hi::marker]
    pub trait Subtype { }
    
    /// Types that qualify group of glsl type.
    #[hi::marker]
    pub trait TypeGroup { }
    
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
    pub trait ForScalar: TransparentType { }

    /// Types valid for use as glsl vectors.
    #[hi::marker]
    pub trait ForVector: TransparentType { }

    /// Types valid for use as glsl matrices.
    #[hi::marker]
    pub trait ForMatrix: TransparentType { }
    
    /// Types valid for use as glsl arrays.
    /// TODO: Check if arrays can indeed store arbitrary types?
    #[hi::marker]
    pub trait ForArray: super::Type { }
    
    /// Types which are valid for use in 
    impl<T> ForScalar for T where T: bounds::ScalarType { }

    /// Any type valid for use as scalar is valid for Vector use.
    impl<T> ForVector for T where T: ForScalar { }
    
    hi::denmark! { f32 as ForMatrix }
    hi::denmark! { f64 as ForMatrix }

    impl<T> ForArray for T where T: Type { }

    pub trait VecDim { }

    hi::denmark! { Const<2> as VecDim }
    hi::denmark! { Const<3> as VecDim }
    hi::denmark! { Const<4> as VecDim }


    // =================[ Opaque types ]================= //

    pub trait ForSampler: OpaqueType { }

    hi::denmark! { f32 as ForSampler }
    hi::denmark! { i32 as ForSampler }
    hi::denmark! { u32 as ForSampler }
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
    T: valid::ForVector,
    Const<SIZE>: valid::VecDim,
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
    T: valid::ForMatrix,
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

/// GLSL array.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Array<T, const N: usize>(PhantomData<T>)
where
    T: Type;

macro_rules! impl_transparent {
    ($ty: ty as $subtype:ident) => {
        impl crate::glsl::Type for $ty {
            type Group = crate::valid::Transparent;
        }
        impl crate::glsl::bounds::TransparentType for $ty {
            type Subtype = crate::valid::$subtype;
        }
    }
}

impl_transparent! { f32 as Scalar }
impl_transparent! { f64 as Scalar }
impl_transparent! { i32 as Scalar }
impl_transparent! { u32 as Scalar }

hi::denmark! { f32 as bounds::ScalarType }
hi::denmark! { f64 as bounds::ScalarType }
hi::denmark! { i32 as bounds::ScalarType }
hi::denmark! { u32 as bounds::ScalarType }

// `Type` impls for Vectors.

impl<T, const N: usize> Type for GVec<T, N>
where
    T: valid::ForVector,
    Const<N>: valid::VecDim,
{
    type Group = valid::Transparent;
}

impl<T, const N: usize> bounds::TransparentType for GVec<T, N>
where 
    T: valid::ForVector,
    Const<N>: valid::VecDim,
{
    type Subtype = valid::Vector;
}

// `Type` impls for Matrices.

impl<T, const R: usize, const C: usize> Type for Mat<T, R, C>
where
    T: valid::ForMatrix,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    type Group = valid::Transparent;
}

impl<T, const R: usize, const C: usize> bounds::TransparentType for Mat<T, R, C>
where
    T: valid::ForMatrix,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    type Subtype = valid::Matrix;
}

// =================[ impl FFI ]================= //

unsafe impl<T, const N: usize> ffi::FFI for GVec<T, N>
where
    T: valid::ForVector,
    Const<N>: valid::VecDim,
{
    type Layout = [T; N];
}

unsafe impl<T, const R: usize, const C: usize> ffi::FFI for Mat<T, R, C>
where
    T: valid::ForMatrix,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{
    type Layout = [[T; C]; R];
}

unsafe impl<T, const N: usize> ffi::FFI for Array<T, N>
where
    T: bounds::TransparentType,
{
    type Layout = [T::Layout; N];
}


// =================[ Opaque types ]================= //

use gl::texture;

pub struct Shadow<Target>(PhantomData<Target>) where Target: texture::Target;

pub struct GSampler<Output, Target>(PhantomData<(Output, Target)>);


type Sampler<Target> = GSampler<Target, f32>;

pub type Sampler1D                = Sampler<texture::target::D1>;
pub type Sampler1DShadow          = Sampler<Shadow<texture::target::D1>>;
pub type Sampler1DArray           = Sampler<texture::target::D1Array>;
pub type Sampler1DArrayShadow     = Sampler<Shadow<texture::target::D1Array>>;
pub type Sampler2D                = Sampler<texture::target::D2>;
pub type Sampler2DShadow          = Sampler<Shadow<texture::target::D2>>;
pub type Sampler2DArray           = Sampler<texture::target::D2Array>;
pub type Sampler2DArrayShadow     = Sampler<Shadow<texture::target::D2Array>>;
pub type Sampler3D                = Sampler<texture::target::D3>;
pub type Sampler2DMS              = Sampler<texture::target::D2MultiSample>;
pub type Sampler2DMSArray         = Sampler<texture::target::D2MultiSampleArray>;
pub type Sampler2DRect            = Sampler<texture::target::Rectangle>;
pub type Sampler2DRectShadow      = Sampler<Shadow<texture::target::Rectangle>>;
pub type Sampler2DCube            = Sampler<texture::target::CubeMap>;
pub type Sampler2DCubeShadow      = Sampler<Shadow<texture::target::CubeMap>>;
pub type Sampler2DCubeArray       = Sampler<texture::target::CubeMapArray>;
pub type Sampler2DCubeArrayShadow = Sampler<Shadow<texture::target::CubeMapArray>>;
pub type SamplerBuffer            = Sampler<texture::target::BUffer>;


type ISampler<Target> = GSampler<Target, i32>;

pub type ISampler1D                = ISampler<texture::target::D1>;
pub type ISampler1DShadow          = ISampler<Shadow<texture::target::D1>>;
pub type ISampler1DArray           = ISampler<texture::target::D1Array>;
pub type ISampler1DArrayShadow     = ISampler<Shadow<texture::target::D1Array>>;
pub type ISampler2D                = ISampler<texture::target::D2>;
pub type ISampler2DShadow          = ISampler<Shadow<texture::target::D2>>;
pub type ISampler2DArray           = ISampler<texture::target::D2Array>;
pub type ISampler2DArrayShadow     = ISampler<Shadow<texture::target::D2Array>>;
pub type ISampler3D                = ISampler<texture::target::D3>;
pub type ISampler2DMS              = ISampler<texture::target::D2MultiSample>;
pub type ISampler2DMSArray         = ISampler<texture::target::D2MultiSampleArray>;
pub type ISampler2DRect            = ISampler<texture::target::Rectangle>;
pub type ISampler2DRectShadow      = ISampler<Shadow<texture::target::Rectangle>>;
pub type ISampler2DCube            = ISampler<texture::target::CubeMap>;
pub type ISampler2DCubeShadow      = ISampler<Shadow<texture::target::CubeMap>>;
pub type ISampler2DCubeArray       = ISampler<texture::target::CubeMapArray>;
pub type ISampler2DCubeArrayShadow = ISampler<Shadow<texture::target::CubeMapArray>>;
pub type ISamplerBuffer            = ISampler<texture::target::BUffer>;


type USampler<Target> = GSampler<Target, u32>;

pub type USampler1D                = USampler<texture::target::D1>;
pub type USampler1DShadow          = USampler<Shadow<texture::target::D1>>;
pub type USampler1DArray           = USampler<texture::target::D1Array>;
pub type USampler1DArrayShadow     = USampler<Shadow<texture::target::D1Array>>;
pub type USampler2D                = USampler<texture::target::D2>;
pub type USampler2DShadow          = USampler<Shadow<texture::target::D2>>;
pub type USampler2DArray           = USampler<texture::target::D2Array>;
pub type USampler2DArrayShadow     = USampler<Shadow<texture::target::D2Array>>;
pub type USampler3D                = USampler<texture::target::D3>;
pub type USampler2DMS              = USampler<texture::target::D2MultiSample>;
pub type USampler2DMSArray         = USampler<texture::target::D2MultiSampleArray>;
pub type USampler2DRect            = USampler<texture::target::Rectangle>;
pub type USampler2DRectShadow      = USampler<Shadow<texture::target::Rectangle>>;
pub type USampler2DCube            = USampler<texture::target::CubeMap>;
pub type USampler2DCubeShadow      = USampler<Shadow<texture::target::CubeMap>>;
pub type USampler2DCubeArray       = USampler<texture::target::CubeMapArray>;
pub type USampler2DCubeArrayShadow = USampler<Shadow<texture::target::CubeMapArray>>;
pub type USamplerBuffer            = USampler<texture::target::BUffer>;