use crate::gl::{self, impl_token};
use crate::prelude::internal::*;
use crate::{ext, gl::types::float16};

use super::image;

pub trait Type: gl::Type {
    type Usage: ty::Usage;
}

/// Implementations of `Type`.
pub mod ty {
    use super::*;
    
    pub trait Usage { }

    #[hi::mark(Usage)]
    /// Usage associated type discriminant for types that contain all pixel channels.
    pub enum Standalone { }

    #[hi::mark(Usage)]
    /// Usage associated type discriminant for types which represent single pixel channel.
    pub enum Aggregate { }
     
    macro_rules! impl_type {
        ($ty:ty => $usage:ty) => {
            impl Type for $ty {
                type Usage = $usage;
            }
        };
    }

    impl_type! { u8      => Aggregate }
    impl_type! { i8      => Aggregate }
    impl_type! { u16     => Aggregate }
    impl_type! { i16     => Aggregate }
    impl_type! { u32     => Aggregate }
    impl_type! { i32     => Aggregate }
    impl_type! { float16 => Aggregate }
    impl_type! { f32     => Aggregate }
}

pub trait Format {
    // Marker for number of components type contains
    type Components: image::format::Components;
    // Marker for sampler output that this type provides
    // Now note how sample Format can provide multiple outputs -- 
    // type Output; -- either geneirc over Output or valid trait?
}

/// Implementation of `Format`.
pub mod format {
    use super::*;
    use crate::prelude::internal::Const;

    use gl::{texture, Type as _};

    impl<T, const N: usize> Format for [T; N]
    where 
        T: Type<Usage=ty::Aggregate>,
        Const<N>: texture::image::format::Components,
    {
        type Components = Const<N>;
        // type Output = T::Kind;
    }
}

/// Implementations of 'Channels`.
pub mod channels {
    use image::format::Components;

    use super::*;
    use crate::gl::impl_token;

    macro_rules! impl_channels {
        ($ty:ty [$components:literal]) => {
            impl Channels for $ty {
                type Components = Const<$components>;
            }
        };
    }
    
    pub trait  Compatible<C: Channels> { }

    pub trait Channels {
        type Components: Components;
    }

    pub enum Red { }
    pub enum Green { }
    pub enum Blue { }
    pub enum RG { }
    pub enum RGB { }
    pub enum BGR { }
    pub enum RGBA { }
    pub enum BGRA { }
    pub enum StencilIndex { }
    pub enum DepthComponent { }
    pub enum DepthStencil { }

    impl_channels! { Red   [1] }
    impl_channels! { Green [1] }
    impl_channels! { Blue  [1] }
    impl_channels! { RG    [2] }
    impl_channels! { RGB   [3] }
    impl_channels! { BGR   [3] }
    impl_channels! { RGBA  [4] }
    impl_channels! { BGRA  [4] }

    // impl_token! { StencilIndex   as Channels => STENCIL_INDEX    }
    // impl_token! { DepthComponent as Channels => DEPTH_COMPONENT  }
    // impl_token! { DepthStencil   as Channels => DEPTH_STENCIL    }
}

pub mod valid {
    use super::*;
    use crate::glsl;

    pub trait ForSamplerOutput<O: glsl::sampler::Output> { }

    /// What target channels are valid for given base format (basically component count of image)
    pub trait ForImageBaseFormat<F: image::marker::BaseFormat> { }

    hi::denmark! { channels::Red as 
        ForImageBaseFormat<image::format::RED>,
        ForImageBaseFormat<image::format::RG>,
        ForImageBaseFormat<image::format::RGB>,
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::Green as
        ForImageBaseFormat<image::format::RG>,
        ForImageBaseFormat<image::format::RGB>,
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::Blue as
        ForImageBaseFormat<image::format::RGB>,
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::RG as
        ForImageBaseFormat<image::format::RG>,
        ForImageBaseFormat<image::format::RGB>,
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::RGB as
        ForImageBaseFormat<image::format::RGB>,
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::BGR as
        ForImageBaseFormat<image::format::RGB>,
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::RGBA as
        ForImageBaseFormat<image::format::RGBA> 
    }

    hi::denmark! { channels::BGRA as
        ForImageBaseFormat<image::format::RGBA> 
    }

    /// Formats that are valid for pixel transfer for given configuration of target channels.
    pub trait ForChannels<Channels: channels::Channels>: Pixel { }
}

pub trait FormatToken {
    const ID: u32;
}

impl_token! { (channels::Red  , gl::types::Float ) as FormatToken =>  RED   }
impl_token! { (channels::Green, gl::types::Float ) as FormatToken =>  GREEN }
impl_token! { (channels::Blue , gl::types::Float ) as FormatToken =>  BLUE  }
impl_token! { (channels::RG   , gl::types::Float ) as FormatToken =>  RG    }
impl_token! { (channels::RGB  , gl::types::Float ) as FormatToken =>  RGB   }
impl_token! { (channels::BGR  , gl::types::Float ) as FormatToken =>  BGR   }
impl_token! { (channels::RGBA , gl::types::Float ) as FormatToken =>  RGBA  }
impl_token! { (channels::BGRA , gl::types::Float ) as FormatToken =>  BGRA  }

impl_token! { (channels::Red  , gl::types::Integer ) as FormatToken =>  RED_INTEGER   }
impl_token! { (channels::Green, gl::types::Integer ) as FormatToken =>  GREEN_INTEGER }
impl_token! { (channels::Blue , gl::types::Integer ) as FormatToken =>  BLUE_INTEGER  }
impl_token! { (channels::RG   , gl::types::Integer ) as FormatToken =>  RG_INTEGER    }
impl_token! { (channels::RGB  , gl::types::Integer ) as FormatToken =>  RGB_INTEGER   }
impl_token! { (channels::BGR  , gl::types::Integer ) as FormatToken =>  BGR_INTEGER   }
impl_token! { (channels::RGBA , gl::types::Integer ) as FormatToken =>  RGBA_INTEGER  }
impl_token! { (channels::BGRA , gl::types::Integer ) as FormatToken =>  BGRA_INTEGER  }

pub trait Pixel: Format {
    type Type: Type;

    fn type_token() -> u32 {
        <Self::Type as gl::Type>::ID
    }
}

impl<T, const N: usize> Pixel for [T; N]
where
    T: Type<Usage=ty::Aggregate>,
    Const<N>: image::format::Components,
    [T; N]: Format,
{
    type Type = T;
}
