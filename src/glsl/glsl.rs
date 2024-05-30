#![allow(unused)]

//! Definitions of glsl types.

use std::marker::PhantomData;

pub mod marker {
    use sealed::sealed;

    use super::Const;
    use crate::glsl::{self, location};
    use crate::ffi;
    use std::marker::PhantomData;

    pub trait VecSize {}

    impl VecSize for Const<2> {}

    impl VecSize for Const<3> {}

    impl VecSize for Const<4> {}

    pub trait Subtype {}
    pub struct Scalar;
    impl Subtype for Scalar {}

    pub struct Vector;
    impl Subtype for Vector {}

    pub struct Matrix;
    impl Subtype for Matrix {}

    #[sealed]
    pub trait TypeGroup {}

    #[derive(Clone, Copy, Debug)]
    pub struct Transparent;
    #[sealed]
    impl TypeGroup for Transparent {}

    #[derive(Clone, Copy, Debug)]
    pub struct Opaque;
    #[sealed]
    impl TypeGroup for Opaque {}

    /// Marker trait for glsl types.
    pub trait Type: location::marker::Location + Default + Clone + Sized + ffi::FFI {
        type Subtype: Subtype;
        type Group: TypeGroup;
    }

    mod impl_type {
        use super::*;
        use crate::glsl;

        /// Marker trait for glsl scalar types.
        impl Type for f32 {
            type Subtype = Scalar;
            type Group = Transparent;
        }

        impl Type for f64 {
            type Subtype = Scalar;
            type Group = Transparent;
        }

        impl Type for i32 {
            type Subtype = Scalar;
            type Group = Transparent;
        }

        impl Type for u32 {
            type Subtype = Scalar;
            type Group = Transparent;
        }

        impl<T, const N: usize> Type for glsl::base::Vec<T, N>
        where
            glsl::base::Vec<T, N>: location::marker::Location,
            T: ScalarType,
            Const<N>: VecSize,
        {
            type Subtype = Vector;
            type Group = Transparent;
        }

        /// Single precision matrix is a valid type.
        impl<const R: usize, const C: usize> Type for glsl::Mat<f32, R, C>
        where
            Const<R>: VecSize,
            Const<C>: VecSize,
        {
            type Subtype = Matrix;
            type Group = Transparent;
        }

        /// Double precision matrix is a valid type.
        impl<const R: usize, const C: usize> Type for glsl::Mat<f64, R, C>
        where
            Const<R>: VecSize,
            Const<C>: VecSize,
        {
            type Subtype = Matrix;
            type Group = Transparent;
        }
    }

    mod impl_ffi {
        use super::*;
        use crate::glsl;

        unsafe impl<T, const N: usize> ffi::FFI for glsl::base::Vec<T, N>
        where
            T: ScalarType,
            Const<N>: VecSize,
        {
            type Layout = [T; N];
        }

        unsafe impl<T, const R: usize, const C: usize> ffi::FFI for glsl::Mat<T, R, C>
        where
            T: ScalarType,
            glsl::Mat<T, R, C>: Type<Subtype = Matrix>,
            Const<R>: VecSize,
            Const<C>: VecSize,
        {
            type Layout = [[T; C]; R];
        }

        unsafe impl<T, const N: usize> ffi::FFI for glsl::Array<T, N>
        where
            T: Type,
        {
            type Layout = [T::Layout; N];
        }

    }

    pub trait ScalarType: Type<Subtype = Scalar> + Clone { }

    pub trait VectorType: Type<Subtype = Vector> {}

    pub trait MatrixType: Type<Subtype = Matrix> {}

    pub trait ArrayType: Type {}

    pub struct Array<S>(PhantomData<S>)
    where
        S: Subtype;
    
    impl<S> Subtype for Array<S> where S: Subtype {}

    impl ScalarType for f32 {}
    impl ScalarType for f64 {}
    impl ScalarType for i32 {}
    impl ScalarType for u32 {}

    impl<T, const N: usize> VectorType for super::base::Vec<T, N>
    where
        super::base::Vec<T, N>: location::marker::Location,
        T: ScalarType,
        Const<N>: VecSize,
    {
    }

    impl<T, const R: usize, const C: usize> MatrixType for super::Mat<T, R, C>
    where
        T: Type<Subtype = Scalar>,
        super::Mat<T, R, C>: Type<Subtype = Matrix>,
        Const<R>: VecSize,
        Const<C>: VecSize,
    {
    }

    /// Array of types is a valid type.
    impl<T, const N: usize> Type for super::Array<T, N>
    where
        T: Type,
    {
        type Subtype = Array<T::Subtype>;
        type Group = Transparent;
    }

    impl<T, const N: usize> ArrayType for super::Array<T, N> where T: Type {}
}

pub use marker::Type;

use crate::ext;

use self::marker::ScalarType;

/// Wrapper for integer values that moves them into type system.
/// Same trick is used in std here `https://doc.rust-lang.org/std/simd/prelude/struct.Simd.html`
pub(crate) struct Const<const NUMBER: usize>;

pub mod base {
    use super::{marker, Const};
    use std::marker::PhantomData;

    /// Generic basis for GLSL Vectors.
    /// GLSL Vectors can contain multiple data types but can only appear in sized of 2, 3 or 4.
    /// This constraint is represented by trait bound `VecSize` on `Const`.
    #[derive(Clone, Debug, Default)]
    pub struct Vec<T, const SIZE: usize>(PhantomData<T>)
    where
        Const<SIZE>: marker::VecSize,
        T: marker::Type;
}

/// Vector of single precision floats.
pub type Vec<const N: usize> = base::Vec<f32, N>;

pub type Vec2 = Vec<2>;
pub type Vec3 = Vec<3>;
pub type Vec4 = Vec<4>;

/// Vector of signed integers.
pub type IVec<const N: usize> = base::Vec<i32, N>;

pub type IVec2 = IVec<2>;
pub type IVec3 = IVec<3>;
pub type IVec4 = IVec<4>;

/// Vector of unsigned integers.
pub type UVec<const N: usize> = base::Vec<u32, N>;

pub type UVec2 = UVec<2>;
pub type UVec3 = UVec<3>;
pub type UVec4 = UVec<4>;
/// Vector of Doubles.
pub type DVec<const N: usize> = base::Vec<f64, N>;

pub type DVec2 = DVec<2>;
pub type DVec3 = DVec<3>;
pub type DVec4 = DVec<4>;

/// SAFETY: note bool here may be ABI incompatible
pub type BVec<const N: usize> = base::Vec<bool, N>;

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
    Const<ROW>: marker::VecSize,
    Const<COL>: marker::VecSize,
    T: marker::Type;

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
    T: marker::Type;
