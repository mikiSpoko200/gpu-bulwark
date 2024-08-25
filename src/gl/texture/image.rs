use crate::prelude::internal::*;
use crate::gl;

pub mod marker {
    use crate::{gl::{self, texture::pixel}, glsl};

    use super::format::Components;

    pub trait BaseFormat {
        const ID: u32;
        type Components: Components;
    }

    pub trait Format {
        const ID: u32;
        type BaseFormat: BaseFormat;
        // Sampler return type for that internalformat 
        type Output: glsl::sampler::Output;
        // Image should know what kind of numbers it expects.
        type Kind: gl::types::Kind;
    }
}

pub struct Format<Components, const BIT_DEPTH: u8, Type = ()>(PhantomData<(Components, Type)>)
where
    Components: format::Components,
    Type: format::Type,
;

pub mod format {
    use super::*;

    pub enum RED { }
    impl marker::BaseFormat for RED { const ID: u32 = glb::RED   ; type Components = components::R; }

    pub struct Rev<T>(PhantomData<T>) where T: Type;

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
        use crate::glsl;

        use super::*;

        pub trait Type {
            type Kind: gl::types::Kind;
            type Output: glsl::sampler::Output;
        }

        impl Type for () { type Kind = gl::types::Integer; type Output = f32; }

        pub enum UI { }
        impl Type for UI { type Kind = gl::types::Integer; type Output = u32; }

        pub enum SNORM { }
        impl Type for SNORM { type Kind = gl::types::Integer; type Output = i32; }

        pub enum I { }
        impl Type for I { type Kind = gl::types::Integer; type Output = f32 ; }

        pub enum F { }
        impl Type for F { type Kind = gl::types::Float; type Output = f32; }
    }

    mod size {
        use super::*;

        #[hi::marker]
        pub trait ValidFor<C: Components, T: Type = ()> { }

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
        ($components:ident, $size:literal, $base:path $(,)?) => {
            ::concat_idents::concat_idents!(token = $components, $size {
                impl marker::Format for Format<components::$components, $size> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = $base;
                    type Output = <() as ty::Type>::Output;
                    type Kind = gl::types::Integer;
                }
            });
        };
        ($components:ident, $size:literal, SNORM, $base:path) => {
            ::concat_idents::concat_idents!(token = $components, $size, _, SNORM {
                impl marker::Format for Format<components::$components, $size, ty::SNORM> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = $base;
                    type Output = i32;
                    type Kind = gl::types::Integer;
                }
            });
        };
        ($components:ident, $size:literal, $ty:ident, $base:path) => {
            ::concat_idents::concat_idents!(token = $components, $size, $ty {
                impl marker::Format for Format<components::$components, $size, ty::$ty> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = $base;
                    type Output = <$ty as ty::Type>::Output;
                    type Kind = gl::types::Integer;
                }
            });
        };
    }

    impl_format! { R,    8              ,   RED     }
    impl_format! { R,    8,       SNORM ,   RED     }
    impl_format! { R,    16             ,   RED     }
    impl_format! { R,    16,      SNORM ,   RED     }
    impl_format! { RG,   8,                 RG      }
    impl_format! { RG,   8,       SNORM ,   RG      }
    impl_format! { RG,   16,                RG      }
    impl_format! { RG,   16,      SNORM ,   RG      }
    impl_format! { RGB,  4,                 RGB     }
    impl_format! { RGB,  5,                 RGB     }
    impl_format! { RGB,  8,                 RGB     }
    impl_format! { RGB,  8,       SNORM ,   RGB     }
    impl_format! { RGB,  10,                RGB     }
    impl_format! { RGB,  12,                RGB     }
    impl_format! { RGB,  16,                RGB     }
    impl_format! { RGB,  16,      SNORM ,   RGB     }
    impl_format! { RGBA, 2,                 RGBA    }
    impl_format! { RGBA, 4,                 RGBA    }
    impl_format! { RGBA, 8,                 RGBA    }
    impl_format! { RGBA, 8,       SNORM ,   RGBA    }
    impl_format! { RGBA, 12,                RGBA    }
    impl_format! { RGBA, 16,                RGBA    }
    impl_format! { RGBA, 16,      SNORM,    RGBA    }
    impl_format! { R,    16,      F,        RED     }
    impl_format! { RG,   16,      F,        RG      }
    impl_format! { RGB,  16,      F,        RGB     }
    impl_format! { RGBA, 16,      F,        RGBA    }
    impl_format! { R,    32,      F,        RED     }
    impl_format! { RG,   32,      F,        RG      }
    impl_format! { RGB,  32,      F,        RGB     }
    impl_format! { RGBA, 32,      F,        RGBA    }
    impl_format! { R,    8,       I,        RED     }
    impl_format! { R,    8,       UI,       RED     }
    impl_format! { R,    16,      I,        RED     }
    impl_format! { R,    16,      UI,       RED     }
    impl_format! { R,    32,      I,        RED     }
    impl_format! { R,    32,      UI,       RED     }
    impl_format! { RG,   8,       I,        RG      }
    impl_format! { RG,   8,       UI,       RG      }
    impl_format! { RG,   16,      I,        RG      }
    impl_format! { RG,   16,      UI,       RG      }
    impl_format! { RG,   32,      I,        RG      }
    impl_format! { RG,   32,      UI,       RG      }
    impl_format! { RGB,  8,       I,        RGB     }
    impl_format! { RGB,  8,       UI,       RGB     }
    impl_format! { RGB,  16,      I,        RGB     }
    impl_format! { RGB,  16,      UI,       RGB     }
    impl_format! { RGB,  32,      I,        RGB     }
    impl_format! { RGB,  32,      UI,       RGB     }
    impl_format! { RGBA, 8,       I,        RGBA    }
    impl_format! { RGBA, 8,       UI,       RGBA    }
    impl_format! { RGBA, 16,      I,        RGBA    }
    impl_format! { RGBA, 16,      UI,       RGBA    }
    impl_format! { RGBA, 32,      I,        RGBA    }
    impl_format! { RGBA, 32,      UI,       RGBA    }
}
