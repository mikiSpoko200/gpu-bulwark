#![allow(unused)]

//! TODO: Add DebugOnly type

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub(crate) struct u31(i32);

impl u31 {
    pub const fn new(inner: i32) -> Self {
        if inner < 0 {
            panic!("value must be non negative")
        }
        Self(inner)
    }

    pub const fn get(self) -> i32 {
        self.0
    }
}

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct irgb10a2(u32);

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct urgb10a2(u32);

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct u10f10f11f(u32);

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct fixed16(u16);

#[repr(transparent)]
#[allow(non_camel_case_types)]
pub struct float16(u16);

// note: There are two kinds of primitive types - primitive types of glsl,
//  and primitive types on the CPU side.
//  GLSL does not understand RGB_10_A_2 type for example. This type along others exists only for
//  storage convenience. Upon transmission to the GPU they are changed to corresponding GLSL types.

/// primitive type compatible with opengl on the ABI layer
/// Known size ang internal layout
/// note: alignment is platform specific
pub unsafe trait Primitive: PartialEq + Copy + Sized + std::fmt::Debug {
    const TYPE_ID: u32;
}

unsafe impl Primitive for u8 {
    const TYPE_ID: u32 = glb::UNSIGNED_BYTE;
}
unsafe impl Primitive for u16 {
    const TYPE_ID: u32 = glb::UNSIGNED_SHORT;
}
unsafe impl Primitive for u32 {
    const TYPE_ID: u32 = glb::UNSIGNED_INT;
}
unsafe impl Primitive for i8 {
    const TYPE_ID: u32 = glb::BYTE;
}
unsafe impl Primitive for i16 {
    const TYPE_ID: u32 = glb::SHORT;
}
unsafe impl Primitive for i32 {
    const TYPE_ID: u32 = glb::INT;
}
unsafe impl Primitive for f32 {
    const TYPE_ID: u32 = glb::FLOAT;
}
unsafe impl Primitive for f64 {
    const TYPE_ID: u32 = glb::DOUBLE;
}