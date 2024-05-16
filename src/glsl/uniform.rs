pub use marker::Uniform;

pub mod signature {
    pub(super) type UniformV<P> = unsafe fn(i32, i32, *const P) -> ();
    pub(super) type UniformMatrixV<P> = unsafe fn(i32, i32, u8, *const P) -> ();
}

mod base {
    use sealed::sealed;
    use crate::glsl;
    use super::signature;

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
                type Signature = signature::UniformV<<$type as glsl::FFI>::Primitive>;
                const FUNCTION: Self::Signature = $function;
            }
        };
        (matrix $type: ty => $function: path) => {
            #[sealed]
            impl Dispatch for $type {
                type Signature = signature::UniformMatrixV<<$type as glsl::FFI>::Primitive>;
                const FUNCTION: Self::Signature = $function;
            }
        };
    }

    dispatch_uniform_functions!{ f32        => gl::Uniform1fv }
    dispatch_uniform_functions!{ glsl::Vec2 => gl::Uniform2fv }
    dispatch_uniform_functions!{ glsl::Vec3 => gl::Uniform3fv }
    dispatch_uniform_functions!{ glsl::Vec4 => gl::Uniform4fv }

    dispatch_uniform_functions!{ i32         => gl::Uniform1iv }
    dispatch_uniform_functions!{ glsl::IVec2 => gl::Uniform2iv }
    dispatch_uniform_functions!{ glsl::IVec3 => gl::Uniform3iv }
    dispatch_uniform_functions!{ glsl::IVec4 => gl::Uniform4iv }

    dispatch_uniform_functions!{ u32         => gl::Uniform1uiv }
    dispatch_uniform_functions!{ glsl::UVec2 => gl::Uniform2uiv }
    dispatch_uniform_functions!{ glsl::UVec3 => gl::Uniform3uiv }
    dispatch_uniform_functions!{ glsl::UVec4 => gl::Uniform4uiv }

    dispatch_uniform_functions!{ f64         => gl::Uniform1dv }
    dispatch_uniform_functions!{ glsl::DVec2 => gl::Uniform2dv }
    dispatch_uniform_functions!{ glsl::DVec3 => gl::Uniform3dv }
    dispatch_uniform_functions!{ glsl::DVec4 => gl::Uniform4dv }

    dispatch_uniform_functions!{ matrix glsl::Mat2x2 => gl::UniformMatrix2fv   }
    dispatch_uniform_functions!{ matrix glsl::Mat2x3 => gl::UniformMatrix2x3fv }
    dispatch_uniform_functions!{ matrix glsl::Mat2x4 => gl::UniformMatrix2x4fv }
    
    dispatch_uniform_functions!{ matrix glsl::Mat3x2 => gl::UniformMatrix3x2fv }
    dispatch_uniform_functions!{ matrix glsl::Mat3x3 => gl::UniformMatrix3fv   }
    dispatch_uniform_functions!{ matrix glsl::Mat3x4 => gl::UniformMatrix3x4fv }
    
    dispatch_uniform_functions!{ matrix glsl::Mat4x2 => gl::UniformMatrix4x2fv }
    dispatch_uniform_functions!{ matrix glsl::Mat4x3 => gl::UniformMatrix4x3fv }
    dispatch_uniform_functions!{ matrix glsl::Mat4x4 => gl::UniformMatrix4fv   }

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

pub mod marker {
    use crate::hlist::lhlist as hlist;
    use crate::glsl::{self, binding};
    use glsl::marker;

    use sealed::sealed;

    /// Uniform must be glsl type and must be a specific subtype
    #[sealed]
    pub trait Uniform: glsl::Type { }

    macro_rules! impl_uniform {
        ($type: ty) => {
            impl UniformDisjointHelper<glsl::marker::Scalar> for $type { }
        };
    }

    impl_uniform!{ f32  }
    impl_uniform!{ i32  }
    impl_uniform!{ u32  }
    impl_uniform!{ bool }
    impl_uniform!{ f64  }

    pub trait UniformDisjointHelper<S> where S: marker::Subtype { }

    impl<U, const SIZE: usize> UniformDisjointHelper<glsl::marker::Vector> for glsl::base::Vec<U, SIZE>
    where
        U: Uniform<Subtype = marker::Scalar>,
        glsl::Const<SIZE>: glsl::VecSize,
        glsl::base::Vec<U, SIZE>: glsl::Type,
    { }

    impl<U, const ROW: usize, const COL: usize> UniformDisjointHelper<glsl::marker::Matrix> for glsl::Mat<U, ROW, COL>
    where
        glsl::Mat<U, ROW, COL>: glsl::Type,
        U: Uniform<Subtype = marker::Scalar>,
        glsl::Const<ROW>: marker::VecSize,
        glsl::Const<COL>: marker::VecSize,
        glsl::base::Vec<U, COL>: glsl::Type,
    { }

    impl<U, S, const N: usize> UniformDisjointHelper<glsl::marker::Array<S>> for glsl::Array<U, N>
    where
        S: glsl::marker::Subtype,
        U: glsl::Uniform<Subtype=S>,
    { }

    #[sealed]
    impl<U, S> Uniform for U
    where
        U: glsl::Type<Subtype=S>,
        U: UniformDisjointHelper<S>,
        S: marker::Subtype
    { }

    /// Marker trait for types that represent program / shader uniforms. 
    #[sealed]
    pub trait Uniforms: hlist::Base { }

    #[sealed]
    impl Uniforms for () { }

    #[sealed]
    impl<H, T, const LOCATION: usize, S> Uniforms for (H, glsl::binding::UniformBinding<T, LOCATION, binding::Validated, S>)
    where
        H: Uniforms,
        T: glsl::Uniform,
        S: binding::Storage,
    { }
}

pub mod ops {
    use crate::gl_call;
    use super::{marker, signature, base};
    use crate::glsl;
    use glsl::binding::UniformBinding;

    pub trait Set<Subtype = <Self as glsl::Type>::Subtype>: marker::Uniform + base::Dispatch
    where
        Subtype: glsl::marker::Subtype,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: glsl::FFI<Primitive = Self::Primitive>,
            GLU: glsl::compatible::Compatible<Self>,
        ;
    }
        
    impl<U> Set<glsl::marker::Scalar> for U
    where
        U: marker::Uniform<Subtype=glsl::marker::Scalar> + base::Dispatch<Signature = signature::UniformV<U::Primitive>>,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: glsl::FFI<Primitive = Self::Primitive> + glsl::compatible::Compatible<Self>,
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
        U: marker::Uniform<Subtype=glsl::marker::Vector> + base::Dispatch<Signature = signature::UniformV<U::Primitive>>,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: glsl::FFI<Primitive = Self::Primitive>,
            GLU: glsl::compatible::Compatible<Self>,
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
        U: marker::Uniform<Subtype=glsl::marker::Matrix> + base::Dispatch<Signature = signature::UniformMatrixV<U::Primitive>>,
    {
        fn set<GLU, const LOCATION: usize>(_: &UniformBinding<Self, LOCATION>, uniform: &GLU)
        where
            GLU: glsl::FFI<Primitive = Self::Primitive>,
            GLU: glsl::compatible::Compatible<Self>,
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