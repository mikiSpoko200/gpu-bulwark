use crate::ext;

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

pub trait FFIExt: FFI {
    fn as_raw_ptr(&self) -> *const <Self::Layout as ext::Array>::Type;
}

impl<T> FFIExt for T where T: FFI {
    fn as_raw_ptr(&self) -> *const <Self::Layout as ext::Array>::Type {
        // SAFETY: unsafe impl for FFI guarantees that self has appropriate binary layout.
        unsafe { std::mem::transmute(self) }
    }
}