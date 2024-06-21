use crate::gl::attributes::{Attribute, AttributeDecl};

use super::prelude::InParameterBinding;
use crate::{glsl, mode};
use crate::ffi;

pub use marker::hlist;
pub use marker::Compatible;

use crate::ext;

mod marker {
    use crate::{ext, glsl};
    use crate::ffi;
    use hi;
 
    /// A GL type is compatible with GLSL type if their `FFI::Layout`s match.
    #[hi::marker]
    pub trait Compatible<GLSL>: ffi::FFI<Layout = GLSL::Layout>
    where
        GLSL: glsl::Type<Group = glsl::marker::Transparent>,
    { }

    pub mod hlist {
        pub trait Compatible<T> {}
    }
}

macro_rules! compatible {
    ($gl: ty => $glsl: path) => {
        impl marker::Compatible<$glsl> for $gl {
        }
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
mod nalgebra_glm {
    use crate::constraint;
    use crate::ffi;
    use crate::glsl;
    use crate::glsl::marker::Matrix;
    use crate::glsl::marker::Scalar;
    use crate::glsl::marker::Vector;
    use crate::glsl::Const;
    use nalgebra_glm as glm;

    unsafe impl<T, const SIZE: usize> ffi::FFI for glm::TVec<T, SIZE>
    where
        T: glsl::Type + constraint::Valid<Vector>,
        Const<SIZE>: constraint::Valid<Vector>,
    {
        type Layout = [T; SIZE];
    }

    impl<T, const SIZE: usize> super::Compatible<glsl::base::Vec<T, SIZE>> for glm::TVec<T, SIZE>
    where
        T: glsl::Type<Group = Scalar> + constraint::Valid<Vector>,
        Const<SIZE>: constraint::Valid<Vector>,
        glsl::base::Vec<T, SIZE>: glsl::location::marker::Location,
        Self: AsRef<Self::Layout>,
    {
        fn as_pod(&self) -> &[T] {
            self.as_ref()
        }
    }

    unsafe impl<T, const ROW: usize, const COL: usize> ffi::FFI for glm::TMat<T, ROW, COL>
    where
        T: constraint::Valid<Matrix>,
        Const<ROW>: constraint::Valid<Vector>,
        Const<COL>: constraint::Valid<Vector>,
    {
        type Layout = [[T; COL]; ROW];
    }

    impl<T, const ROW: usize, const COL: usize> super::Compatible<glsl::Mat<T, ROW, COL>> for glm::TMat<T, ROW, COL>
    where
        T: constraint::Valid<Matrix>,
        Const<ROW>: constraint::Valid<Vector>,
        Const<COL>: constraint::Valid<Vector>,
        glsl::Mat<T, ROW, COL>: glsl::Type<Layout = [[T; COL]; ROW]>,
        Self: AsRef<[[T; COL]; ROW]>,
    {
        fn as_pod(&self) -> &[T] {
            unsafe { std::slice::from_raw_parts(self.as_ref().as_ptr() as *const _, ROW * COL) }
        }
    }
}

// --------==========[ Arrays of Compatible types ]==========--------

impl<GL, GLSL, const N: usize> marker::Compatible<glsl::Array<GLSL, N>> for &GL
where
    GL: Compatible<GLSL>,
    GLSL: super::Type<Layout = GL::Layout>,
{
    fn as_pod(&self) -> &[<GLSL::Layout as ext::Array>::Type] {
        unsafe {
            std::slice::from_raw_parts(
                <Self as marker::Compatible<glsl::glsl::Array<GLSL, N>>>::as_pod(self).as_ptr()
                    as *const _,
                GLSL::SIZE,
            )
        }
    }
}

/// NOTE: This won't work sin
impl<GLSL, GL, const N: usize> marker::Compatible<glsl::Array<GLSL, N>> for [GL; N]
where
    GL: Compatible<GLSL>,
    GLSL: super::Type<Layout = GL::Layout>,
{
    fn as_pod(&self) -> &[<GLSL::Layout as ext::Array>::Type] {
        unsafe {
            std::slice::from_raw_parts(
                <Self as marker::Compatible<glsl::glsl::Array<GLSL, N>>>::as_pod(self).as_ptr()
                    as *const _,
                GLSL::SIZE,
            )
        }
    }
}

// --------==========[ HList integration ]==========--------

impl marker::hlist::Compatible<()> for () {}

impl<'buffers, AS, A, PS, P, const ATTRIBUTE_INDEX: usize>
    marker::hlist::Compatible<(PS, InParameterBinding<P, ATTRIBUTE_INDEX>)>
    for (AS, AttributeDecl<'buffers, A, ATTRIBUTE_INDEX>)
where
    A: Attribute + ffi::FFI<Layout = P::Layout>,
    A: marker::Compatible<P>,
    AS: marker::hlist::Compatible<PS>,
    P: glsl::Type + mode::Validation,
{
}
