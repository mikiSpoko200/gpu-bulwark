#![allow(unused)]

//! Definitions of glsl types.

use std::marker::PhantomData;

pub mod marker {
    use super::Const;
    use crate::glsl::{self, location};
    use crate::{constraint, ffi};
    use crate::mode;
    use std::marker::PhantomData;
    use hi;

    impl constraint::Valid<Vector> for Const<2> {}

    impl constraint::Valid<Vector> for Const<3> {}

    impl constraint::Valid<Vector> for Const<4> {}

    mod sealed {
        #[hi::marker]
        pub trait Subtype { }
        
        #[hi::marker]
        pub trait TypeGroup {}
    }

    pub use sealed::*;

    #[hi::mark(Subtype, mode::Validation)]
    pub enum Scalar { }

    #[hi::mark(Subtype, mode::Validation)]
    pub enum Vector { }

    #[hi::mark(Subtype, mode::Validation)]
    pub enum Matrix { }
    
    #[derive(Clone, Copy, Debug)]
    #[hi::mark(sealed::TypeGroup)]
    pub struct Transparent;

    #[derive(Clone, Copy, Debug)]
    #[hi::mark(TypeGroup)]
    pub struct Opaque;

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
            T: glsl::Type + constraint::Valid<Vector>,
            Const<N>: constraint::Valid<Vector>,
        {
            type Subtype = Vector;
            type Group = Transparent;
        }

        impl<T, const R: usize, const C: usize> Type for glsl::Mat<T, R, C>
        where
            T: constraint::Valid<Matrix>,
            Const<R>: constraint::Valid<Vector>,
            Const<C>: constraint::Valid<Vector>,
        {
            type Subtype = Matrix;
            type Group = Transparent;
        }
    }

    mod impl_ffi {
        use super::*;
        use crate::{constraint, ext, glsl};

        unsafe impl<T, const N: usize> ffi::FFI for glsl::base::Vec<T, N>
        where
            T: Type + constraint::Valid<Vector> + ext::Array,
            Const<N>: constraint::Valid<Vector>,
        {
            type Layout = [T; N];
        }

        unsafe impl<T, const R: usize, const C: usize> ffi::FFI for glsl::Mat<T, R, C>
        where
            T: glsl::Type + constraint::Valid<Matrix>,
            Const<R>: constraint::Valid<Vector>,
            Const<C>: constraint::Valid<Vector>,
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

    // pub trait ScalarType: Type<Subtype = Scalar, Group = Transparent> { }

    // pub trait VectorType: Type<Subtype = Vector, Group = Transparent> {}

    // pub trait MatrixType: Type<Subtype = Matrix, Group = Transparent> {}

    pub trait ArrayType: Type {}

    pub struct Array<S>(PhantomData<S>)
    where
        S: Subtype;
    
    impl<S> Subtype for Array<S> where S: Subtype {}

    impl<T> constraint::Valid<Scalar> for T where T: Type<Subtype = Scalar, Group = Transparent> {}

    /// Any type valid for use as scalar is valid for Vector use.
    impl<T> constraint::Valid<Vector> for T where T: constraint::Valid<Scalar> { }

    impl constraint::Valid<Matrix> for f32 { }
    impl constraint::Valid<Matrix> for f64 { }

    impl<T, const N: usize> Type for super::Array<T, N>
    where
        T: Type,
    {
        type Subtype = Array<T::Subtype>;
        type Group = Transparent;
    }

    impl<T, const N: usize> ArrayType for super::Array<T, N> where T: Type {}
}

use marker::{Matrix, Vector};
pub use marker::Type;

use crate::{constraint, ext};

/// Wrapper for integer values that moves them into type system.
/// Same trick is used in std here `https://doc.rust-lang.org/std/simd/prelude/struct.Simd.html`
pub(crate) struct Const<const NUMBER: usize>;

pub mod base {
    use super::{marker::{self, Vector}, Const};
    use std::marker::PhantomData;
    use crate::constraint;

    /// Generic basis for GLSL Vectors.
    /// GLSL Vectors can contain multiple data types but can only appear in sized of 2, 3 or 4.
    /// This constraint is represented by trait bound `VecSize` on `Const`.
    #[derive(Clone, Debug, Default)]
    pub struct Vec<T, const SIZE: usize>(PhantomData<T>)
    where
        Const<SIZE>: constraint::Valid<Vector>,
        T: marker::Type + constraint::Valid<Vector>;
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
    Const<ROW>: constraint::Valid<Vector>,
    Const<COL>: constraint::Valid<Vector>,
    T: marker::Type + constraint::Valid<Matrix>;

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
