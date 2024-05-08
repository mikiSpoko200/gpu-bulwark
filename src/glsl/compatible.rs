use crate::object::attributes::{AttributeDecl, Attribute};
use crate::object::buffer::target;

use super::marker::ScalarType;
use super::prelude::InParameterBinding;
use crate::glsl;

pub use marker::Compatible;
pub use marker::hlist;

mod marker {
    use crate::glsl;

    pub trait Compatible<T>: Clone + Sized + glsl::FFI 
    where
        T: glsl::FFI<Primitive=Self::Primitive>,
    {
        const CHECK_SAME_SIZE: ();
        fn as_pod(&self) -> &[T::Primitive];
    }
    
    pub mod hlist {
        pub trait Compatible<T> { }
    }
}

macro_rules! compatible {
    (scalar $gl: ty => $glsl: path) => {
        impl marker::Compatible<$glsl> for $gl {
            const CHECK_SAME_SIZE: () = assert!({<$gl as glsl::FFI>::SIZE} == {<$glsl as glsl::FFI>::SIZE});
            fn as_pod(&self) -> &[<$glsl as glsl::FFI>::Primitive] {
                std::slice::from_ref(self)
            }
        }
    };
    ($gl: ty => $glsl: path) => {
        impl marker::Compatible<$glsl> for $gl {
            const CHECK_SAME_SIZE: () = assert!(<$gl as glsl::FFI>::SIZE == <$glsl as glsl::FFI>::SIZE);
            fn as_pod(&self) -> &[<$glsl as glsl::FFI>::Primitive] {
                self
            }
        }
    };
}

unsafe impl<S, const N: usize> glsl::FFI for [S; N]
where
    S: glsl::marker::ScalarType,
{
    type Primitive = S;

    const SIZE: usize = N;
}

// Base Types
compatible!{ scalar f32 => f32 }
compatible!{ scalar f64 => f64 }
compatible!{ scalar i32 => i32 }
compatible!{ scalar u32 => u32 }

compatible!{ [f32; 2] => glsl::Vec2 }
compatible!{ [f32; 3] => glsl::Vec3 }
compatible!{ [f32; 4] => glsl::Vec4 }

compatible!{ [i32; 2] => glsl::IVec2 }
compatible!{ [i32; 3] => glsl::IVec3 }
compatible!{ [i32; 4] => glsl::IVec4 }

compatible!{ [u32; 2] => glsl::UVec2 }
compatible!{ [u32; 3] => glsl::UVec3 }
compatible!{ [u32; 4] => glsl::UVec4 }

compatible!{ [f64; 2] => glsl::DVec2 }
compatible!{ [f64; 3] => glsl::DVec3 }
compatible!{ [f64; 4] => glsl::DVec4 }

// OpenGL bools are wierd (32 bit not `_Bool`)
// compatible!([i32; 2], glsl::BVec2);
// compatible!([i32; 3], glsl::BVec3);
// compatible!([i32; 4], glsl::BVec4);
// compatible!([u32; 2], glsl::BVec2);
// compatible!([u32; 3], glsl::BVec3);
// compatible!([u32; 4], glsl::BVec4);

compatible!{ [f32; 4] => glsl::Mat2x2 }
compatible!{ [f32; 6] => glsl::Mat2x3 }
compatible!{ [f32; 8] => glsl::Mat2x4 }

compatible!{ [f32; 6]  => glsl::Mat3x2 }
compatible!{ [f32; 9]  => glsl::Mat3x3 }
compatible!{ [f32; 12] => glsl::Mat3x4 }

compatible!{ [f32; 8]  => glsl::Mat4x2 }
compatible!{ [f32; 12] => glsl::Mat4x3 }
compatible!{ [f32; 16] => glsl::Mat4x4 }

compatible!{ [f64; 4] => glsl::DMat2x2 }
compatible!{ [f64; 6] => glsl::DMat2x3 }
compatible!{ [f64; 8] => glsl::DMat2x4 }

compatible!{ [f64; 6]  => glsl::DMat3x2 }
compatible!{ [f64; 9]  => glsl::DMat3x3 }
compatible!{ [f64; 12] => glsl::DMat3x4 }

compatible!{ [f64; 8]  => glsl::DMat4x2 }
compatible!{ [f64; 12] => glsl::DMat4x3 }
compatible!{ [f64; 16] => glsl::DMat4x4 }

impl<GL, GLSL, const N: usize> marker::Compatible<glsl::Array<GLSL, N>> for [GL; N]
where
    GL: ScalarType,
    GLSL: super::Type<Primitive=GL>,
{
    const CHECK_SAME_SIZE: () = assert!(<[GL; N] as glsl::FFI>::SIZE == <glsl::Array<GLSL, N> as glsl::FFI>::SIZE);
    fn as_pod(&self) -> &[GLSL::Primitive] {
        unsafe { std::slice::from_raw_parts(self as *const _, GLSL::SIZE) }
    }
}

// Lists of types - base case
impl marker::hlist::Compatible<()> for () { }

// List of types - inductive step
impl<'buffers, AS, A, PS, P, const ATTRIBUTE_INDEX: usize> marker::hlist::Compatible<(PS, InParameterBinding<P, ATTRIBUTE_INDEX>)> for (AS, AttributeDecl<'buffers, A, ATTRIBUTE_INDEX>)
where
    A: Attribute + glsl::FFI<Primitive = <P as glsl::FFI>::Primitive>,
    A: marker::Compatible<P>,
    AS: marker::hlist::Compatible<PS>,
    (target::Array, A): target::format::Valid,
    P: glsl::Type,
{ }
