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

pub unsafe trait Type {
    const ID: u32;
}

pub trait Packed: Type { }

unsafe impl Type for u8  { const ID: u32 = glb::UNSIGNED_BYTE;  }
unsafe impl Type for u16 { const ID: u32 = glb::UNSIGNED_SHORT; }
unsafe impl Type for u32 { const ID: u32 = glb::UNSIGNED_INT;   }
unsafe impl Type for i8  { const ID: u32 = glb::BYTE;           }
unsafe impl Type for i16 { const ID: u32 = glb::SHORT;          }
unsafe impl Type for i32 { const ID: u32 = glb::INT;            }

// unsafe impl Type for irgb10a2   { const ID: u32 = glb::INT_2_10_10_10_REV;           }
// unsafe impl Type for urgb10a2   { const ID: u32 = glb::UNSIGNED_INT_2_10_10_10_REV;  }
// unsafe impl Type for u10f10f11f { const ID: u32 = glb::UNSIGNED_INT_10F_11F_11F_REV; }

// hi::denmark! { irgb10a2   as Packed }
// hi::denmark! { urgb10a2   as Packed }
// hi::denmark! { u10f10f11f as Packed }

unsafe impl Type for fixed16 { const ID: u32 = glb::FIXED;      }
unsafe impl Type for float16 { const ID: u32 = glb::HALF_FLOAT; }
unsafe impl Type for f32 { const ID: u32 = glb::FLOAT;          }
unsafe impl Type for f64 { const ID: u32 = glb::DOUBLE;         }

pub struct Normalized<I>(I);
