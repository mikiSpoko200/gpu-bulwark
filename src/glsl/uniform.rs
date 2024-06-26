pub use marker::Uniform;



pub mod signature {
    pub(super) type UniformV<P> = unsafe fn(i32, i32, *const P) -> ();
    pub(super) type UniformMatrixV<P> = unsafe fn(i32, i32, u8, *const P) -> ();
}

mod base {
    use super::signature;
    use crate::glsl;
    use crate::ffi;
    use sealed::sealed;

    use super::marker::Uniform;

    #[sealed]
    pub trait Dispatch {
        type Signature;
        const FUNCTION: Self::Signature;
        const COUNT: usize = 1;
    }

    macro_rules! dispatch_uniform_functions {
        ($type: ty => $function: path) => {
            #[sealed]
            impl Dispatch for $type {
                type Signature = signature::UniformV<<$type as ffi::FFI>::Layout>;
                const FUNCTION: Self::Signature = $function;
            }
        };
        (matrix $type: ty => $function: path) => {
            impl sealed::Sealed for $type { }
            impl Dispatch for $type {
                type Signature = signature::UniformMatrixV<<$type as ffi::FFI>::Layout>;
                const FUNCTION: Self::Signature = $function;
            }
        };
    }

    dispatch_uniform_functions! { f32        => glb::Uniform1fv }
    dispatch_uniform_functions! { glsl::Vec2 => glb::Uniform2fv }
    dispatch_uniform_functions! { glsl::Vec3 => glb::Uniform3fv }
    dispatch_uniform_functions! { glsl::Vec4 => glb::Uniform4fv }

    dispatch_uniform_functions! { i32         => glb::Uniform1iv }
    dispatch_uniform_functions! { glsl::IVec2 => glb::Uniform2iv }
    dispatch_uniform_functions! { glsl::IVec3 => glb::Uniform3iv }
    dispatch_uniform_functions! { glsl::IVec4 => glb::Uniform4iv }

    dispatch_uniform_functions! { u32         => glb::Uniform1uiv }
    dispatch_uniform_functions! { glsl::UVec2 => glb::Uniform2uiv }
    dispatch_uniform_functions! { glsl::UVec3 => glb::Uniform3uiv }
    dispatch_uniform_functions! { glsl::UVec4 => glb::Uniform4uiv }

    dispatch_uniform_functions! { f64         => glb::Uniform1dv }
    dispatch_uniform_functions! { glsl::DVec2 => glb::Uniform2dv }
    dispatch_uniform_functions! { glsl::DVec3 => glb::Uniform3dv }
    dispatch_uniform_functions! { glsl::DVec4 => glb::Uniform4dv }

    dispatch_uniform_functions! { matrix glsl::Mat2x2 => glb::UniformMatrix2fv   }
    dispatch_uniform_functions! { matrix glsl::Mat2x3 => glb::UniformMatrix2x3fv }
    dispatch_uniform_functions! { matrix glsl::Mat2x4 => glb::UniformMatrix2x4fv }

    dispatch_uniform_functions! { matrix glsl::Mat3x2 => glb::UniformMatrix3x2fv }
    dispatch_uniform_functions! { matrix glsl::Mat3x3 => glb::UniformMatrix3fv   }
    dispatch_uniform_functions! { matrix glsl::Mat3x4 => glb::UniformMatrix3x4fv }

    dispatch_uniform_functions! { matrix glsl::Mat4x2 => glb::UniformMatrix4x2fv }
    dispatch_uniform_functions! { matrix glsl::Mat4x3 => glb::UniformMatrix4x3fv }
    dispatch_uniform_functions! { matrix glsl::Mat4x4 => glb::UniformMatrix4fv   }

    #[sealed]
    impl<U, const N: usize> Dispatch for glsl::Array<U, N>
    where
        U: Uniform + Dispatch,
    {
        type Signature = U::Signature;
        const FUNCTION: Self::Signature = U::FUNCTION;
        const COUNT: usize = U::COUNT * N;
    }
}

/// Uniform must be glsl type and must be a specific subtype
#[hi::marker]
pub trait Uniform: super::Type { }

pub mod alias {
    use crate::{constraint, glsl, valid};
    use crate::hlist::lhlist as hlist;
    use crate::mode;

    pub trait TransparentUniform: Uniform<Group = valid::Transparent> { }
    pub trait OpaqueUniform: Uniform<Group = valid::Transparent> { }

    macro_rules! impl_uniform {
        ($type: ty) => {
            impl UniformDisjointHelper<valid::Scalar> for $type { }
        };
    }

    impl_uniform! { f32  }
    impl_uniform! { i32  }
    impl_uniform! { u32  }
    impl_uniform! { bool }
    impl_uniform! { f64  }

    pub trait UniformDisjointHelper<S>
    where
        S: valid::Subtype,
    {
    }

    impl<U, const SIZE: usize> UniformDisjointHelper<valid::Vector> for glsl::GVec<U, SIZE>
    where
        U: Uniform<Subtype = glsl::Scalar>,
        glsl::Const<SIZE>: constraint::Valid<Vector>,
        glsl::base::Vec<U, SIZE>: glsl::Type,
    {
    }

    impl<U, const ROW: usize, const COL: usize> UniformDisjointHelper<glsl::marker::Matrix>
        for glsl::Mat<U, ROW, COL>
    where
        glsl::Mat<U, ROW, COL>: glsl::Type,
        U: Uniform<Subtype = glsl::marker::Scalar, Group = Transparent> + constraint::Valid<Matrix>,
        glsl::Const<ROW>: constraint::Valid<Vector>,
        glsl::Const<COL>: constraint::Valid<Vector>,
        glsl::base::Vec<U, COL>: glsl::Type + constraint::Valid<Vector>,
    {
    }

    impl<U, S, const N: usize> UniformDisjointHelper<glsl::marker::Array<S>> for glsl::Array<U, N>
    where
        S: glsl::marker::Subtype,
        U: glsl::Uniform<Subtype = S>,
    {
    }

    #[sealed]
    impl<U, S> Uniform for U
    where
        U: glsl::Type<Subtype = S>,
        U: UniformDisjointHelper<S>,
        S: glsl::marker::Subtype,
    { }

    /// Marker trait for types that represent program / shader uniforms.
    #[sealed]
    pub trait Uniforms: hlist::Base {}

    #[sealed]
    impl Uniforms for () {}

    #[sealed]
    impl<H, T, const LOCATION: usize, S> Uniforms for (H, glsl::binding::UniformBinding<T, LOCATION, S>)
    where
        H: Uniforms,
        T: glsl::Uniform,
        S: mode::Storage,
    { }
}

pub mod valid {
    
}

pub mod ops {
    use super::{base, marker, signature};
    use crate::{glsl, ext, gl_call, ffi};
    use glsl::binding::UniformBinding;

    pub trait Set<Subtype = <Self as glsl::Type>::Subtype>: marker::Uniform<Group = glsl::marker::Transparent> + base::Dispatch
    where
        Subtype: glsl::marker::Subtype,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: ffi::FFI<Layout = Self::Layout>,
            GLU: glsl::Compatible<Self>;
    }

    impl<U> Set<glsl::marker::Scalar> for U
    where
        U: marker::Uniform<Subtype = glsl::marker::Scalar, Group = glsl::marker::Transparent>
            + base::Dispatch<Signature = signature::UniformV<<U::Layout as ext::Array>::Type>>,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: ffi::FFI<Layout = Self::Layout> + glsl::Compatible<Self>,
        {
            gl_call! {
                #[panic]
                unsafe {
                    Self::FUNCTION(LOCATION as _, Self::COUNT as _, uniform.as_pod().as_ptr());
                }
            };
        }
    }

    impl<U> Set<glsl::marker::Vector> for U
    where
        U: marker::Uniform<Subtype = glsl::marker::Vector, Group = glsl::marker::Transparent>
            + base::Dispatch<Signature = signature::UniformV<<U::Layout as ext::Array>::Type>>,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: glsl::Compatible<Self>,
        {
            gl_call! {
                #[panic]
                unsafe {
                    Self::FUNCTION(LOCATION as _, Self::COUNT as _, uniform.as_pod().as_ptr());
                }
            };
        }
    }

    impl<U> Set<glsl::marker::Matrix> for U
    where
        U: marker::Uniform<Subtype = glsl::marker::Matrix, Group = glsl::marker::Transparent>
            + base::Dispatch<Signature = signature::UniformMatrixV<<U::Layout as ext::Array>::Type>>,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: glsl::Compatible<Self>,
        {
            gl_call! {
                #[panic]
                unsafe {
                    Self::FUNCTION(LOCATION as _, Self::COUNT as _, true as _, uniform.as_pod().as_ptr());
                }
            };
        }
    }
}

#[derive(Clone)]
pub struct Definition<const INDEX: usize, U>(pub U) where U: marker::Uniform;

#[derive(Clone)]
pub struct Definitions<US>(pub US);

impl Definitions<()> {
    pub fn new() -> Self {
        Self(())
    }
}

impl Default for Definitions<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DUS> Definitions<DUS> {
    pub fn define<const INDEX: usize, U>(self, u: U) -> Definitions<(DUS, Definition<INDEX, U>)>
    where
        U: marker::Uniform
    {
        Definitions((self.0, Definition(u)))
    }
}
