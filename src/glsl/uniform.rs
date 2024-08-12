use crate::prelude::internal::*;

use crate::glsl;
use crate::hlist::lhlist as hlist;
use crate::md;
use crate::ext;

use glsl::valid;

use crate::ffi;

/// Uniforms from the perspective of the OpenGL differ in how user interacts with them. Namely there exist a designated setter function for each type of transparent uniform.
/// This presents unique challenge when trying to abstract away setting of a uniform since different types have different function signatures.
/// Additionally there is some regularity amongst functions and they can be grouped by number of components in a case for vector types, as well as, by 
/// Fortunately there exists bijective correspondence between types and functions.

/// Dispatch for OpenGL uniform setters for uniform types.
pub trait DispatchSetters {
    /// Type specific setter signature
    type Signature;
    /// Pointer to OpenGL uniform setter function
    const SETTER: Self::Signature;
    /// Number of elements that setter sets, 1 for single values, >1 for arrays.
    const N_ELEMENTS: usize = 1;
}

macro_rules! dispatch_uniform_setters {
    ($type: ty => $function: path) => {
        impl DispatchSetters for $type {
            type Signature = signature::UniformV<<<$type as ffi::FFI>::Layout as ext::Array>::Type>;
            const SETTER: Self::Signature = $function;
        }
    };
    ([matrix] $type: ty => $function: path) => {
        impl DispatchSetters for $type {
            type Signature = signature::UniformMatrixV<<<$type as ffi::FFI>::Layout as ext::Array>::Type>;
            const SETTER: Self::Signature = $function;
        }
    };
}

dispatch_uniform_setters! { f32        => glb::Uniform1fv }
dispatch_uniform_setters! { glsl::Vec2 => glb::Uniform2fv }
dispatch_uniform_setters! { glsl::Vec3 => glb::Uniform3fv }
dispatch_uniform_setters! { glsl::Vec4 => glb::Uniform4fv }

dispatch_uniform_setters! { i32         => glb::Uniform1iv }
dispatch_uniform_setters! { glsl::IVec2 => glb::Uniform2iv }
dispatch_uniform_setters! { glsl::IVec3 => glb::Uniform3iv }
dispatch_uniform_setters! { glsl::IVec4 => glb::Uniform4iv }

dispatch_uniform_setters! { u32         => glb::Uniform1uiv }
dispatch_uniform_setters! { glsl::UVec2 => glb::Uniform2uiv }
dispatch_uniform_setters! { glsl::UVec3 => glb::Uniform3uiv }
dispatch_uniform_setters! { glsl::UVec4 => glb::Uniform4uiv }

dispatch_uniform_setters! { f64         => glb::Uniform1dv }
dispatch_uniform_setters! { glsl::DVec2 => glb::Uniform2dv }
dispatch_uniform_setters! { glsl::DVec3 => glb::Uniform3dv }
dispatch_uniform_setters! { glsl::DVec4 => glb::Uniform4dv }

dispatch_uniform_setters! { [matrix] glsl::Mat2x2 => glb::UniformMatrix2fv   }
dispatch_uniform_setters! { [matrix] glsl::Mat2x3 => glb::UniformMatrix2x3fv }
dispatch_uniform_setters! { [matrix] glsl::Mat2x4 => glb::UniformMatrix2x4fv }

dispatch_uniform_setters! { [matrix] glsl::Mat3x2 => glb::UniformMatrix3x2fv }
dispatch_uniform_setters! { [matrix] glsl::Mat3x3 => glb::UniformMatrix3fv   }
dispatch_uniform_setters! { [matrix] glsl::Mat3x4 => glb::UniformMatrix3x4fv }

dispatch_uniform_setters! { [matrix] glsl::Mat4x2 => glb::UniformMatrix4x2fv }
dispatch_uniform_setters! { [matrix] glsl::Mat4x3 => glb::UniformMatrix4x3fv }
dispatch_uniform_setters! { [matrix] glsl::Mat4x4 => glb::UniformMatrix4fv   }

/// Delegate setter to inner type, only increase the number of elements to be set.
impl<U, const N: usize> DispatchSetters for glsl::Array<U, N>
where
    U: Uniform + DispatchSetters,
{
    type Signature = U::Signature;
    const SETTER: Self::Signature = U::SETTER;
    const N_ELEMENTS: usize = U::N_ELEMENTS * N;
}
/// Uniform must be glsl type and must be a specific subtype
#[hi::marker]
pub trait Uniform: super::Type { }

pub trait UniformDH<S>: Uniform
where
    S: valid::Subtype,
{ }

impl<T> UniformDH<valid::Scalar> for T where T: bounds::ScalarUniform { }

impl<U, const DIM: usize> UniformDH<valid::Vector<DIM>> for glsl::GVec<U, DIM>
where
    U: valid::ForVector<DIM>,
    Const<DIM>: valid::VecDim,
{ }

impl<U, const R: usize, const C: usize> UniformDH<valid::Matrix> for glsl::Mat<U, R, C>
where
    U: valid::ForMatrix<R, C>,
    Const<R>: valid::VecDim,
    Const<C>: valid::VecDim,
{ }

impl<U, S, const N: usize> UniformDH<valid::Array<S>> for glsl::Array<U, N>
where
    S: valid::Subtype,
    U: glsl::uniform::bounds::TransparentUniform<Subtype = S>,
{ }

impl<U, S> Uniform for U
where
    U: glsl::bounds::TransparentType<Subtype = S>,
    U: UniformDH<S>,
    S: valid::Subtype,
{ }

/// Marker trait for types that represent program / shader uniforms.
pub trait Uniforms: hlist::Base { }

impl Uniforms for () { }

impl<H, T, const LOCATION: usize, S> Uniforms for (H, glsl::variable::UniformVariable<T, LOCATION, S>)
where
    H: Uniforms,
    T: glsl::Uniform,
    S: md::Storage,
{ }

pub mod signature {
    pub(super) type UniformV<P> = unsafe fn(i32, i32, *const P) -> ();
    pub(super) type UniformMatrixV<P> = unsafe fn(i32, i32, u8, *const P) -> ();
}

pub mod bounds {
    use super::*;

    pub trait TransparentUniform: Uniform + glsl::bounds::TransparentType + ops::Set { }

    impl<T> TransparentUniform for T where T: Uniform + glsl::bounds::TransparentType + ops::Set { }

    pub trait OpaqueUniform: Uniform + glsl::bounds::OpaqueType { }

    pub trait ScalarUniform: TransparentUniform + glsl::bounds::ScalarType
        + DispatchSetters<Signature = signature::UniformV<<Self::Layout as ext::Array>::Type>>
    { }

    impl<T> ScalarUniform for T 
    where 
        T: TransparentUniform + glsl::bounds::ScalarType
        + DispatchSetters<Signature = signature::UniformV<<Self::Layout as ext::Array>::Type>>
    { }

    pub trait VectorUniform<const DIM: usize>: TransparentUniform + glsl::bounds::VectorType<DIM>
        + DispatchSetters<Signature = signature::UniformV<<Self::Layout as ext::Array>::Type>>
    where Const<DIM>: valid::VecDim { }

    impl<T, const DIM: usize> VectorUniform<DIM> for T
    where 
        Const<DIM>: valid::VecDim,
        T: TransparentUniform + glsl::bounds::VectorType<DIM>
        + DispatchSetters<Signature = signature::UniformV<<Self::Layout as ext::Array>::Type>>
    { }
    
    pub trait MatrixUniform: TransparentUniform + glsl::bounds::MatrixType
    + DispatchSetters<Signature = signature::UniformMatrixV<<Self::Layout as ext::Array>::Type>>
    { }
    
    impl<T> MatrixUniform for T where T: TransparentUniform + glsl::bounds::MatrixType
    + DispatchSetters<Signature = signature::UniformMatrixV<<Self::Layout as ext::Array>::Type>>
    { }
}

/// # Capabilities for uniform types
/// 
/// ## Notes: 
/// 
/// - single blanket impl is impossible due to differences in signatures.
pub mod ops {
    use super::*;

    use crate::ffi;
    use crate::gl;
    use glsl::variable::UniformVariable;
    use ffi::FFIExt;

    pub trait Set<Subtype = <Self as glsl::bounds::TransparentType>::Subtype>: glsl::bounds::TransparentType + DispatchSetters
    where
        Subtype: valid::Subtype,
    {
        fn set<const LOCATION: usize>(_: &UniformVariable<Self, LOCATION>, uniform: &impl glsl::Compatible<Self>);
    }

    impl<U> Set<valid::Scalar> for U
    where
        U: glsl::bounds::TransparentType + Uniform
        + DispatchSetters<Signature = signature::UniformV<<U::Layout as ext::Array>::Type>>,
    {
        fn set<const LOCATION: usize>(_: &UniformVariable<Self, LOCATION>, uniform: &impl glsl::Compatible<Self>) {
            gl::call! {
                [panic]
                unsafe {
                    Self::SETTER(LOCATION as _, <Self as glsl::Location>::N_USED_LOCATIONS as _, uniform.as_slice().as_ptr());
                }
            };
        }
    }

    impl<U, const DIM: usize> Set<valid::Vector<DIM>> for U
    where
        U: glsl::bounds::TransparentType + Uniform
        + DispatchSetters<Signature = signature::UniformV<<U::Layout as ext::Array>::Type>>,
        Const<DIM>: valid::VecDim,
    {
        fn set<const LOCATION: usize>(_: &UniformVariable<Self, LOCATION>, uniform: &impl glsl::Compatible<Self>) {
            gl::call! {
                [panic]
                unsafe {
                    Self::SETTER(LOCATION as _, Self::N_ELEMENTS as _, uniform.as_slice().as_ptr());
                }
            };
        }
    }

    impl<U> Set<valid::Matrix> for U
    where
        U: glsl::bounds::TransparentType + Uniform
        + DispatchSetters<Signature = signature::UniformMatrixV<<U::Layout as ext::Array>::Type>>,
    {
        fn set<const LOCATION: usize>(_: &UniformVariable<Self, LOCATION>, uniform: &impl glsl::Compatible<Self>) {
            gl::call! {
                [panic]
                unsafe {
                    Self::SETTER(LOCATION as _, Self::N_ELEMENTS as _, true as _, uniform.as_slice().as_ptr());
                }
            };
        }
    }
}
