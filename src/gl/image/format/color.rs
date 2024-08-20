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


pub mod valid {
    use super::*;

    #[hi::marker]
    pub trait For<C: Components> { }
}

pub enum BitDepth<const N: u8> { }

pub struct Format<C: Components>(PhantomData<C>);