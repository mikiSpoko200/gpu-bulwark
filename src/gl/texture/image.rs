use crate::prelude::internal::*;
use crate::gl;

pub mod marker {
    use super::*;

    use crate::{gl::{self, texture::pixel}, glsl};

    use super::format::Components;

    pub trait BaseFormat {
        const ID: u32;
        type Components: Components;
    }

    pub trait FormatType { }

    
    #[hi::mark(FormatType)]
    pub enum Aggregate { }
    
    #[hi::mark(FormatType)]
    pub enum Special { }

    pub trait Format {
        const ID: u32;
        type Output: glsl::sampler::Output;
        type Composition: FormatType;
        type BaseFormat: BaseFormat;
        type ComponentType: gl::Type;
    }

    pub trait AggregateFormat: Format<Composition = Aggregate> { }
}

pub struct Format<Components, ComponentType, Interpretation = format::UNorm>(PhantomData<(Components, ComponentType, Interpretation)>)
where
    Components: format::Components,
    Interpretation: format::Interpretation,
;

pub mod format {
    use super::*;

    macro_rules! map_base_format {
        (R) => { RED };
        ($ty:ty) => { $ty };
    }

    pub enum RED { }
    impl marker::BaseFormat for RED { const ID: u32 = glb::RED   ; type Components = components::R; }

    pub struct Rev<T>(PhantomData<T>) where T: Interpretation;

    pub use components::{Components, RG, RGB, RGBA};
    pub use ty::*;

    pub mod components {
        use super::*;

        /// Count of color components given type provides -- used in `Format`.
        #[hi::marker]
        pub trait Components { }

        hi::denmark! { Const<1> as Components }
        hi::denmark! { Const<2> as Components }
        hi::denmark! { Const<3> as Components }
        hi::denmark! { Const<4> as Components }

        pub type R = Const<1>;
        pub type RG = Const<2>;
        pub type RGB = Const<3>;
        pub type RGBA = Const<4>;
        
        impl marker::BaseFormat for RG { const ID: u32 = glb::RG    ; type Components = Self; }
        impl marker::BaseFormat for RGB { const ID: u32 = glb::RGB  ; type Components = Self; }
        impl marker::BaseFormat for RGBA { const ID: u32 = glb::RGBA ; type Components = Self; }
    }

    pub mod ty {
        use gl::Integer;

        use crate::glsl;

        use super::*;

        pub trait Interpretation {
            /// Kind of number produced by sampler in s shader.
            type Output: glsl::sampler::Output;
        }

        // Floats
        pub enum F { }
        impl Interpretation for F { type Output = f32; }

        // Integers 
        pub enum UI { }
        impl Interpretation for UI { type Output = u32; }
        pub enum I { }
        impl Interpretation for I { type Output = i32; }

        // Integers to be normalized
        pub struct Norm<I>(PhantomData<I>) where I: Interpretation, I::Output: glsl::sampler::Output<Kind=Integer>;
        
        impl<I> Interpretation for Norm<I> where I: Interpretation, I::Output: glsl::sampler::Output<Kind=Integer> {
            type Output = f32;
        }
        
        pub type SNorm = Norm<I>;
        pub type UNorm = Norm<UI>;

        // #[macro_export]
        macro_rules! suffix_to_type {
            () => { UNorm };
            (_SNORM) => { SNorm };
            ($ty:ty) => { $ty }
        }
        pub(super) use suffix_to_type;
    }

    mod size {
        use super::*;

        #[hi::marker]
        pub trait ValidFor<C: Components, T: Interpretation = UNorm> { }

        // impls for unsigned normalized format
        hi::denmark! { Const<2> as ValidFor<RGBA> }
        hi::denmark! { Const<4> as ValidFor<RGBA>, ValidFor<RGB> }
        hi::denmark! { Const<12> as ValidFor<RGBA>, ValidFor<RGB> }
        hi::denmark! { Const<5> as ValidFor<RGB> }
        hi::denmark! { Const<10> as ValidFor<RGB> }

        // impls for signed normalized format

        // impls for unsigned normalized format
        hi::denmark! { Const<8> as ValidFor<RGBA, > }
        hi::denmark! { Const<16> as ValidFor<RGBA>, ValidFor<RGB> }
    }

    macro_rules! impl_format {
        ($components:ident, $component_type:ty, $size:literal) => {
            ::concat_idents::concat_idents!(token = $components, $size {
                impl marker::Format for Format<components::$components, $component_type, suffix_to_type!()> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = map_base_format!($components);
                    type ComponentType = $component_type;
                    type Composition = marker::Aggregate;
                    type Output = <ty::suffix_to_type!() as ty::Interpretation>::Output;
                    // type Kind = gl::types::Integer;
                }
            });
        };
        ($components:ident, $component_type:ty, $size:literal, $ty_suffix:ident) => {
            ::concat_idents::concat_idents!(token = $components, $size, $ty_suffix {
                impl marker::Format for Format<components::$components, $component_type, suffix_to_type!($ty_suffix)> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = map_base_format!($components);
                    type ComponentType = $component_type;
                    type Composition = marker::Aggregate;
                    type Output = <suffix_to_type!($ty_suffix) as ty::Interpretation>::Output;
                    // type Kind = gl::types::Integer;
                }
            });
        };
    }

    impl_format! { R,    u8,    8            }
    impl_format! { R,    i8,    8,    _SNORM }
    impl_format! { R,    u16,   16           }
    impl_format! { R,    i16,   16,   _SNORM }
    impl_format! { RG,   u8,    8            }
    impl_format! { RG,   i8,    8,    _SNORM }
    impl_format! { RG,   u16,   16           }
    impl_format! { RG,   i16,   16,   _SNORM }
    // impl_format! { RGB,  4               }
    // impl_format! { RGB,  5               }
    impl_format! { RGB,  u8, 8              }
    impl_format! { RGB,  i8,    8,    _SNORM }
    // impl_format! { RGB,  10              }
    // impl_format! { RGB,  12              }
    impl_format! { RGB,  u16, 16             }
    impl_format! { RGB,  i16, 16,     _SNORM }
    // impl_format! { RGBA, 2               }
    // impl_format! { RGBA, 4               }
    impl_format! { RGBA, u8, 8              }
    impl_format! { RGBA, i8, 8,      _SNORM }
    // impl_format! { RGBA, 12              }
    impl_format! { RGBA, u16, 16             }
    impl_format! { RGBA, i16, 16,     _SNORM }
    impl_format! { R,    gl::types::float16, 16,     F      }
    impl_format! { RG,   gl::types::float16, 16,     F      }
    impl_format! { RGB,  gl::types::float16, 16,     F      }
    impl_format! { RGBA, gl::types::float16, 16,     F      }
    impl_format! { R,    f32, 32,  F     }
    impl_format! { RG,   f32, 32,  F     }
    impl_format! { RGB,  f32, 32,  F     }
    impl_format! { RGBA, f32, 32,  F     }
    impl_format! { R,    u8, 8,    I      }
    impl_format! { R,    i8, 8,    UI     }
    impl_format! { R,    u16, 16,  I      }
    impl_format! { R,    i16, 16,  UI     }
    impl_format! { R,    u32, 32,  I      }
    impl_format! { R,    i32, 32,  UI     }
    impl_format! { RG,   u8, 8,       I      }
    impl_format! { RG,   i8, 8,       UI     }
    impl_format! { RG,   u16, 16,      I      }
    impl_format! { RG,   i16, 16,      UI     }
    impl_format! { RG,   u32, 32,      I      }
    impl_format! { RG,   i32, 32,      UI     }
    impl_format! { RGB,  u8,  8,     I      }
    impl_format! { RGB,  i8,  8,     UI     }
    impl_format! { RGB,  u16,  16,    I      }
    impl_format! { RGB,  i16,  16,    UI     }
    impl_format! { RGB,  u32,  32,    I      }
    impl_format! { RGB,  i32,  32,    UI     }
    impl_format! { RGBA, u8,  8,     I      }
    impl_format! { RGBA, i8,  8,     UI     }
    impl_format! { RGBA, u16,   16,   I      }
    impl_format! { RGBA, i16,   16,   UI     }
    impl_format! { RGBA, u32,   32,   I      }
    impl_format! { RGBA, i32,   32,   UI     }
}
