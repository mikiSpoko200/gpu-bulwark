use crate::gl::attributes::{Attribute, AttributeDecl};
use crate::prelude::internal::*;

use crate::glsl;
use glsl::binding::InParameterBinding;
use crate::ffi;
use crate::valid;

/// A GL type is compatible with GLSL type if their `FFI::Layout`s match.
#[hi::marker]
pub trait Compatible<GLSL>: ffi::FFI<Layout = GLSL::Layout>
where
    GLSL: glsl::bounds::TransparentType,
{ }

pub mod hlist {
    pub trait Compatible<T> {}
}

macro_rules! compatible {
    ($gl: ty => $glsl: path) => {
        hi::denmark! { $gl as Compatible<$glsl> }
    };
}

// --------==========[ Rust base types ]==========--------

/// This is causing more trouble then its worth
unsafe impl<S, const N: usize> ffi::FFI for [S; N]
where
    S: ffi::FFI,
{
    type Layout = Self;
}

compatible! { f32 => f32 }
compatible! { f64 => f64 }
compatible! { i32 => i32 }
compatible! { u32 => u32 }

compatible! { [f32; 2] => glsl::Vec2 }
compatible! { [f32; 3] => glsl::Vec3 }
compatible! { [f32; 4] => glsl::Vec4 }

compatible! { [i32; 2] => glsl::IVec2 }
compatible! { [i32; 3] => glsl::IVec3 }
compatible! { [i32; 4] => glsl::IVec4 }

compatible! { [u32; 2] => glsl::UVec2 }
compatible! { [u32; 3] => glsl::UVec3 }
compatible! { [u32; 4] => glsl::UVec4 }

compatible! { [f64; 2] => glsl::DVec2 }
compatible! { [f64; 3] => glsl::DVec3 }
compatible! { [f64; 4] => glsl::DVec4 }

compatible! { [[f32; 2]; 2] => glsl::Mat2x2 }
compatible! { [[f32; 3]; 2] => glsl::Mat2x3 }
compatible! { [[f32; 4]; 2] => glsl::Mat2x4 }

compatible! { [[f32; 2]; 3] => glsl::Mat3x2 }
compatible! { [[f32; 3]; 3] => glsl::Mat3x3 }
compatible! { [[f32; 4]; 3] => glsl::Mat3x4 }

compatible! { [[f32; 2]; 4] => glsl::Mat4x2 }
compatible! { [[f32; 3]; 4] => glsl::Mat4x3 }
compatible! { [[f32; 4]; 4] => glsl::Mat4x4 }

compatible! { [[f64; 2]; 2] => glsl::DMat2x2 }
compatible! { [[f64; 3]; 2] => glsl::DMat2x3 }
compatible! { [[f64; 4]; 2] => glsl::DMat2x4 }

compatible! { [[f64; 2]; 3] => glsl::DMat3x2 }
compatible! { [[f64; 3]; 3] => glsl::DMat3x3 }
compatible! { [[f64; 4]; 3] => glsl::DMat3x4 }

compatible! { [[f64; 2]; 4] => glsl::DMat4x2 }
compatible! { [[f64; 3]; 4] => glsl::DMat4x3 }
compatible! { [[f64; 4]; 4] => glsl::DMat4x4 }

// --------==========[ nalgebra types ]==========--------

#[cfg(feature = "nalgebra")]
mod nalgebra {
    // TODO: implement this
}

#[cfg(feature = "nalgebra-glm")]
mod impl_nalgebra_glm {
    use super::*;

    use ::nalgebra_glm as glm;

    unsafe impl<T, const SIZE: usize> ffi::FFI for glm::TVec<T, SIZE>
    where
        T: valid::ForVector,
        Const<SIZE>: valid::ForVector,
    {
        type Layout = [T; SIZE];
    }

    impl<T, const SIZE: usize> super::Compatible<glsl::GVec<T, SIZE>> for glm::TVec<T, SIZE>
    where
        T: valid::ForVector,
        Const<SIZE>: valid::ForVector,
        Self: AsRef<Self::Layout>,
    { }

    unsafe impl<T, const ROW: usize, const COL: usize> ffi::FFI for glm::TMat<T, ROW, COL>
    where
        T: valid::ForMatrix,
        Const<ROW>: valid::ForVector,
        Const<COL>: valid::ForVector,
    {
        type Layout = [[T; COL]; ROW];
    }

    impl<T, const ROW: usize, const COL: usize> super::Compatible<glsl::Mat<T, ROW, COL>> for glm::TMat<T, ROW, COL>
    where
        T: valid::ForMatrix,
        Const<ROW>: valid::ForVector,
        Const<COL>: valid::ForVector,
    { }
}

// --------==========[ Arrays of Compatible types ]==========--------

impl<GL, GLSL, const N: usize> Compatible<glsl::Array<GLSL, N>> for &GL
where
    GL: Compatible<GLSL>,
    GLSL: glsl::bounds::TransparentType<Layout = GL::Layout>,
{ }

/// NOTE: This won't work sin
impl<GLSL, GL, const N: usize> Compatible<glsl::Array<GLSL, N>> for [GL; N]
where
    GL: Compatible<GLSL>,
    GLSL: glsl::bounds::TransparentType<Layout = GL::Layout>,
{ }

// --------==========[ HList integration ]==========--------

impl hlist::Compatible<()> for () {}

impl<'buffers, AS, A, PS, P, const ATTRIBUTE_INDEX: usize>
    hlist::Compatible<(PS, InParameterBinding<P, ATTRIBUTE_INDEX>)>
    for (AS, AttributeDecl<'buffers, A, ATTRIBUTE_INDEX>)
where
    A: Attribute,
    A: Compatible<P>,
    AS: hlist::Compatible<PS>,
{ }
