use crate::prelude::internal::*;
use crate::gl;


pub mod marker {
    pub trait BaseFormat {
        const ID: u32;
    }

    pub trait Format {
        const ID: u32;
        type BaseFormat: BaseFormat;
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

    gl::impl_token! { RED as marker::BaseFormat => RED }

    pub struct Rev<T>(PhantomData<T>) where T: Type;

    pub use components::*;
    pub use ty::*;

    pub mod components {
        use super::*;

        pub trait Components {
            type DerivedBaseFormat: marker::BaseFormat;
        }

        pub enum R { }
        impl Components for R {
            type DerivedBaseFormat = RED;
        }

        pub enum RG { }
        gl::impl_token! { RG as marker::BaseFormat => RG }
        impl Components for RG {
            type DerivedBaseFormat = Self;
        }

        pub enum RGB { }
        gl::impl_token! { RGB as marker::BaseFormat => RGB }
        impl Components for RGB {
            type DerivedBaseFormat = Self;
        }

        pub enum RGBA { }
        gl::impl_token! { RGBA as marker::BaseFormat => RGBA }
        impl Components for RGBA {
            type DerivedBaseFormat = Self;
        }
    }

    pub mod ty {
        use super::*;

        pub trait Type { }

        hi::denmark! { () as Type }

        #[hi::mark(Type)]
        pub enum UI { }

        #[hi::mark(Type)]
        pub enum SNORM { }

        #[hi::mark(Type)]
        pub enum I { }

        #[hi::mark(Type)]
        pub enum F { }
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
                }
            });
        };
        ($components:ident, $size:literal, SNORM, $base:path) => {
            ::concat_idents::concat_idents!(token = $components, $size, _, SNORM {
                impl marker::Format for Format<components::$components, $size, ty::SNORM> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = $base;
                }
            });
        };
        ($components:ident, $size:literal, $ty:ident, $base:path) => {
            ::concat_idents::concat_idents!(token = $components, $size, $ty {
                impl marker::Format for Format<components::$components, $size, ty::$ty> {
                    const ID: u32 = ::glb::token;
                    type BaseFormat = $base;
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


//                
//       
//                
//       
//                 
//        
//             
//        
//            
//            
//            
//       
//            
//            
//            
//       
//           
//           
//           
//     
//           
//           
// 
//            
//             
//            
//           
//            
//             
//            
//         
//                
//            
//            
//           
//            
//           
//             
//           
//             
//           
//             
//           
//            
//          
//            
//          
//            
//          
//           
//         
//           
//     
//           
// ,    32,       UI