use crate::prelude::internal::*;

use crate::gl::image::format;


pub mod marker {
    use super::*;

    pub trait Format {
        type Components: Components;
        type Type: format::Type;
        type BitDepth<const N: u8>: valid::For<Self::Components>;
    }
}

#[hi::marker]
pub trait Components { }

#[hi::mark(Components)]
pub enum RED { }

#[hi::mark(Components)]
pub enum RG { }

#[hi::mark(Components)]
pub enum RGB { }

#[hi::mark(Components)]
pub enum RGBA { }


pub mod valid {
    use super::*;

    #[hi::marker]
    pub trait For<C: Components> { }
}

pub enum BitDepth<const N: u8> { }

pub struct Format<C: Components>(PhantomData<C>);