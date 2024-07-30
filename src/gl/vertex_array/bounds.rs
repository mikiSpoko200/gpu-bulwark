use crate::prelude::internal::*;

use crate::gl;
use crate::glsl;

pub trait AttribFormat {
    type Type: gl::Type;
    const N_COMPONENTS: usize;
}

impl<T, const N: usize> AttribFormat for [T; N] where Const<N>: glsl::valid::VecDim {
    const N_COMPONENTS: usize = N;
    type Type = f32;
}

impl AttribFormat for gl::types::u10f10f11f {
    type Type = f32;
    const N_COMPONENTS: usize = 4;
}

impl AttribFormat for gl::types::urgb10a2 {
    type Type = f32;
    const N_COMPONENTS: usize = 4;
}

impl AttribFormat for gl::types::irgb10a2 {
    type Type = f32;
    const N_COMPONENTS: usize = 4;
}

#[hi::marker]
pub trait Format: AttribFormat { }


impl<const N: usize> Format for [f32; N] where Const<N>: glsl::valid::VecDim { }

hi::denmark! { gl::types::u10f10f11f as Format }
hi::denmark! { gl::types::irgb10a2 as Format }
hi::denmark! { gl::types::urgb10a2 as Format }

#[hi::marker]
pub trait IFormat: AttribFormat { }

impl<T, const N: usize> IFormat for [T; N] where [T; N]: AttribFormat, Const<N>: glsl::valid::VecDim { }

#[hi::marker]
pub trait LFormat: AttribFormat { }

impl<const N: usize> LFormat for [f32; N] where Const<N>: glsl::valid::VecDim { }

pub mod valid {
    use super::*;

    use gl::types::{fixed16, float16, u10f10f11f, irgb10a2, urgb10a2};

    pub trait ForFormatBase: gl::Type { }

    pub trait ForFormat: ForFormatBase { }

    hi::denmark! { i8  as ForFormat, ForFormatBase }
    hi::denmark! { i16 as ForFormat, ForFormatBase }
    hi::denmark! { i32 as ForFormat, ForFormatBase }
    hi::denmark! { u8  as ForFormat, ForFormatBase }
    hi::denmark! { u16 as ForFormat, ForFormatBase }
    hi::denmark! { u32 as ForFormat, ForFormatBase }

    hi::denmark! { f32 as ForFormat, ForFormatBase }
    hi::denmark! { f64 as ForFormat, ForFormatBase }
    hi::denmark! { fixed16 as ForFormat, ForFormatBase }
    hi::denmark! { float16 as ForFormat, ForFormatBase }
    
    // hi::denmark! { u10f10f11f as ForFormat } 
    // hi::denmark! { irgb10a2   as ForFormat } 
    // hi::denmark! { urgb10a2   as ForFormat } 

    pub trait ForIFormat: ForFormatBase { }

    hi::denmark! { i8  as ForIFormat }
    hi::denmark! { i16 as ForIFormat }
    hi::denmark! { i32 as ForIFormat }
    hi::denmark! { u8  as ForIFormat }
    hi::denmark! { u16 as ForIFormat }
    hi::denmark! { u32 as ForIFormat }

    pub trait ForLFormat: ForFormatBase { }

    hi::denmark! { f64 as ForLFormat }
}