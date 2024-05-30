use crate::ext;

/// Array storage that type can be transmutated into
pub unsafe trait FFI {
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