use crate::gl;
use crate::prelude::internal::*;

use crate::glsl;
use crate::ffi;

use glsl::valid;

/// A GL type is compatible with GLSL type if their `FFI::Layout`s match.
pub trait Compatible<GLSL>: ffi::FFI<Layout = GLSL::Layout>
where
    GLSL: glsl::bounds::TransparentType,
{ }

impl<GL, GLSL> Compatible<GLSL> for GL where GL: ffi::FFI<Layout = GLSL::Layout>, GLSL: glsl::bounds::TransparentType { }

pub mod hlist {
    use crate::{gl, glsl};

    pub trait Compatible<T>: gl::vertex_array::valid::Attributes
    where
        T: glsl::Parameters<glsl::storage::In>
    { }
}

macro_rules! compatible {
    ($gl:ty => $glsl:path) => {
        hi::denmark! { $gl as Compatible<$glsl> }
    };
}

// compatible! { f32 => f32 }
// compatible! { f64 => f64 }
// compatible! { i32 => i32 }
// compatible! { u32 => u32 }

// compatible! { [f32; 2] => glsl::Vec2 }
// compatible! { [f32; 3] => glsl::Vec3 }
// compatible! { [f32; 4] => glsl::Vec4 }

// compatible! { [i32; 2] => glsl::IVec2 }
// compatible! { [i32; 3] => glsl::IVec3 }
// compatible! { [i32; 4] => glsl::IVec4 }

// compatible! { [u32; 2] => glsl::UVec2 }
// compatible! { [u32; 3] => glsl::UVec3 }
// compatible! { [u32; 4] => glsl::UVec4 }

// compatible! { [f64; 2] => glsl::DVec2 }
// compatible! { [f64; 3] => glsl::DVec3 }
// compatible! { [f64; 4] => glsl::DVec4 }

// compatible! { [[f32; 2]; 2] => glsl::Mat2x2 }
// compatible! { [[f32; 3]; 2] => glsl::Mat2x3 }
// compatible! { [[f32; 4]; 2] => glsl::Mat2x4 }

// compatible! { [[f32; 2]; 3] => glsl::Mat3x2 }
// compatible! { [[f32; 3]; 3] => glsl::Mat3x3 }
// compatible! { [[f32; 4]; 3] => glsl::Mat3x4 }

// compatible! { [[f32; 2]; 4] => glsl::Mat4x2 }
// compatible! { [[f32; 3]; 4] => glsl::Mat4x3 }
// compatible! { [[f32; 4]; 4] => glsl::Mat4x4 }

// compatible! { [[f64; 2]; 2] => glsl::DMat2x2 }
// compatible! { [[f64; 3]; 2] => glsl::DMat2x3 }
// compatible! { [[f64; 4]; 2] => glsl::DMat2x4 }

// compatible! { [[f64; 2]; 3] => glsl::DMat3x2 }
// compatible! { [[f64; 3]; 3] => glsl::DMat3x3 }
// compatible! { [[f64; 4]; 3] => glsl::DMat3x4 }

// compatible! { [[f64; 2]; 4] => glsl::DMat4x2 }
// compatible! { [[f64; 3]; 4] => glsl::DMat4x3 }
// compatible! { [[f64; 4]; 4] => glsl::DMat4x4 }

// --------==========[ nalgebra types ]==========--------

#[cfg(feature = "nalgebra")]
mod nalgebra {
    // TODO: implement this
}

#[cfg(feature = "nalgebra-glm")]
mod impl_nalgebra_glm {
    use super::*;
    use ::nalgebra_glm as glm;

    unsafe impl<T, const DIM: usize> ffi::FFI for glm::TVec<T, DIM>
    where
        T: valid::ForVector<DIM>,
        Const<DIM>: valid::VecDim,
    {
        type Layout = [T::Layout; DIM];
    }

    // impl<T, const DIM: usize> super::Compatible<glsl::GVec<T, DIM>> for glm::TVec<T, DIM>
    // where
    //     T: valid::ForVector<DIM>,
    //     Const<DIM>: valid::VecDim,
    //     Self: AsRef<Self::Layout>,
    // { }

    unsafe impl<T, const R: usize, const C: usize> ffi::FFI for glm::TMat<T, R, C>
    where
        T: valid::ForMatrix<R, C>,
        Const<R>: valid::VecDim,
        Const<C>: valid::VecDim,
    {
        type Layout = [[T::Layout; C]; R];
    }

    // impl<T, const R: usize, const C: usize> super::Compatible<glsl::Mat<T, R, C>> for glm::TMat<T, R, C>
    // where
    //     T: valid::ForMatrix<R, C>,
    //     Const<R>: valid::VecDim,
    //     Const<C>: valid::VecDim,
    // { }
}

// --------==========[ Arrays of Compatible types ]==========--------

// impl<GL, GLSL, const N: usize> Compatible<glsl::Array<GLSL, N>> for &GL
// where
//     GL: Compatible<GLSL>,
//     GLSL: glsl::bounds::TransparentType,
// { }

// impl<GLSL, GL, const N: usize> Compatible<glsl::Array<GLSL, N>> for [GL; N]
// where
//     GL: Compatible<GLSL>,
//     GLSL: glsl::bounds::TransparentType,
// { }

// --------==========[ HList integration ]==========--------

impl hlist::Compatible<()> for () {}

use crate::gl::vertex_array as vao;
use vao::attribute::Attribute;

impl<'buffer, PH, AH, GLSL, GL, const ATTRIB_INDEX: usize> hlist::Compatible<(PH, glsl::InVariable<GLSL, ATTRIB_INDEX>)> for (AH, Attribute<GL, ATTRIB_INDEX>)
where
    GLSL: glsl::parameters::Parameter<glsl::storage::In>,
    PH: glsl::parameters::Parameters<glsl::storage::In>,
    AH: vao::valid::Attributes,
    GL: vao::bounds::AttribFormat,
    AH: hlist::Compatible<PH>,
{ }
