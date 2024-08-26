#![allow(unused)]

// TODO: Implement different types using bitfields crate

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

#[hi::marker]
pub trait Kind { }

#[hi::mark(Kind)]
pub enum Integer { }

#[hi::mark(Kind)]
pub enum Float { }

pub unsafe trait Type {
    const ID: u32;
    type Kind;
}

pub trait Packed: Type { }

unsafe impl Type for u8  { const ID: u32 = glb::UNSIGNED_BYTE   ; type Kind = Integer; }
unsafe impl Type for u16 { const ID: u32 = glb::UNSIGNED_SHORT  ; type Kind = Integer; }
unsafe impl Type for u32 { const ID: u32 = glb::UNSIGNED_INT    ; type Kind = Integer; }
unsafe impl Type for i8  { const ID: u32 = glb::BYTE            ; type Kind = Integer; }
unsafe impl Type for i16 { const ID: u32 = glb::SHORT           ; type Kind = Integer; }
unsafe impl Type for i32 { const ID: u32 = glb::INT             ; type Kind = Integer; }

unsafe impl Type for fixed16 { const ID: u32 = glb::FIXED       ; type Kind = Float; }
unsafe impl Type for float16 { const ID: u32 = glb::HALF_FLOAT  ; type Kind = Float; }
unsafe impl Type for f32     { const ID: u32 = glb::FLOAT       ; type Kind = Float; }
unsafe impl Type for f64     { const ID: u32 = glb::DOUBLE      ; type Kind = Float; }

#[repr(transparent)]
pub struct Normalized<I>(I);
