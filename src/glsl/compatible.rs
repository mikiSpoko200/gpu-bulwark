use crate::object::attributes::{AttributeDecl, Attribute};
use crate::object::buffer::target;

use super::types;

pub unsafe trait Compatible<A, T> {}

macro_rules! compatible {
    ($gl: ty, $glsl: path) => {
        unsafe impl Compatible<$gl, $glsl> for ($gl, $glsl) {}
    };
    ($gl: ty, $glsl: ty) => {
        unsafe impl Compatible<$gl, $glsl> for ($gl, $glsl) {}
    };
}

// Base Types
compatible!(f32, f32);
compatible!(f64, f64);
compatible!(i32, i32);
compatible!(u32, u32);
compatible!(bool, bool);

compatible!([f32; 2], types::Vec2);
compatible!([f32; 3], types::Vec3);
compatible!([f32; 4], types::Vec4);

compatible!([i32; 2], types::IVec2);
compatible!([i32; 3], types::IVec3);
compatible!([i32; 4], types::IVec4);

compatible!([u32; 2], types::UVec2);
compatible!([u32; 3], types::UVec3);
compatible!([u32; 4], types::UVec4);

compatible!([f64; 2], types::DVec2);
compatible!([f64; 3], types::DVec3);
compatible!([f64; 4], types::DVec4);

compatible!([bool; 2], types::BVec2);
compatible!([bool; 3], types::BVec3);
compatible!([bool; 4], types::BVec4);

compatible!([f32; 4], types::Mat2x2);
compatible!([f32; 6], types::Mat2x3);
compatible!([f32; 8], types::Mat2x4);

compatible!([f32; 6], types::Mat3x2);
compatible!([f32; 9], types::Mat3x3);
compatible!([f32; 12], types::Mat3x4);

compatible!([f32; 8], types::Mat4x2);
compatible!([f32; 12], types::Mat4x3);
compatible!([f32; 16], types::Mat4x4);

compatible!([f64; 4], types::DMat2x2);
compatible!([f64; 6], types::DMat2x3);
compatible!([f64; 8], types::DMat2x4);

compatible!([f64; 6], types::DMat3x2);
compatible!([f64; 9], types::DMat3x3);
compatible!([f64; 12], types::DMat3x4);

compatible!([f64; 8], types::DMat4x2);
compatible!([f64; 12], types::DMat4x3);
compatible!([f64; 16], types::DMat4x4);

unsafe impl<Gl, Glsl, const N: usize> Compatible<[Gl; N], types::Array<Glsl, N>> for ([Gl; N], types::Array<Glsl, N>) {}

// Lists of types - base case
compatible!((), ());

// List of types - inductive step
unsafe impl<'buffers, AS, A, PS, P, const ATTRIBUTE_INDEX: usize> Compatible<(AS, AttributeDecl<'buffers, A, ATTRIBUTE_INDEX>), (PS, P)>
    for ((AS, AttributeDecl<'buffers, A, ATTRIBUTE_INDEX>), (PS, P))
where
    A: Attribute,
    (A, P): Compatible<A, P>,
    (AS, PS): Compatible<AS, PS>,
    (target::Array, A): target::format::Valid,
    P: types::Type,
{
}
