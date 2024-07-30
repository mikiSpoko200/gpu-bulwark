use crate::{ext, glsl};

/// Array storage that type can be transmuted into
pub unsafe trait FFI: Sized {
    type Layout: ext::Array;
}

unsafe impl FFI for f32 {
    type Layout = Self;
}

unsafe impl FFI for f64 {
    type Layout = Self;
}

unsafe impl FFI for i32 {
    type Layout = Self;
}

unsafe impl FFI for u32 {
    type Layout = Self;
}

unsafe impl<S, const N: usize> FFI for [S; N]
where
    S: FFI,
{
    type Layout = [S::Layout; N];
}

pub const fn as_slice<GLSL, GL>(value: &GL) -> &[<GL::Layout as ext::Array>::Type]
where
    GLSL: glsl::bounds::TransparentType,
    GL: glsl::Compatible<GLSL>,
{
    let raw_pointer = unsafe { (value as *const GL).cast::<<GL::Layout as ext::Array>::Type>() };
    unsafe {
        std::slice::from_raw_parts(raw_pointer, <GL::Layout as ext::Array>::SIZE)
    }
}

pub trait FFIExt: FFI {
    fn as_slice(&self) -> &[<Self::Layout as ext::Array>::Type];
}

impl<T> FFIExt for T where T: FFI {
    fn as_slice(&self) -> &[<Self::Layout as ext::Array>::Type] {
        // SAFETY: unsafe impl for FFI guarantees that self has appropriate binary layout.
        let raw_pointer = unsafe { std::mem::transmute::<&T, *const <T::Layout as ext::Array>::Type>(self) };
        unsafe {
            std::slice::from_raw_parts(raw_pointer, <T::Layout as ext::Array>::SIZE)
        }
    }
}