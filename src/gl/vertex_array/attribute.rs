//! This module provides specialization of HLists for Vertex Array Object attributes.

use nalgebra_glm::Scalar;

use crate::gl::buffer;
use crate::constraint;
use crate::ffi;
use crate::glsl;
use crate::valid;
use crate::md;
use crate::types;
use crate::hlist;

use crate::prelude::internal::*;

pub(crate) struct Attribute<'buffer, A, const INDEX: usize>
where
    A: valid::ForAttribute
{
    pub buffer: &'buffer buffer::Buffer<buffer::Array, A>,
}

pub mod _valid {
    use super::*;

    pub trait ForAttribute: valid::ForBuffer<buffer::Array> {
        const N_COMPONENTS: usize;
    }

    #[hi::marker]
    pub(crate) trait Attributes: crate::hlist::lhlist::Base { }

    impl Attributes for () { }
}

impl<T> valid::ForAttribute for T where T: valid::ForBuffer<buffer::Array> { }

impl<'buffer, A, AS, const INDEX: usize> valid::Attributes for (AS, Attribute<'buffer, A, INDEX>)
where
    A: valid::ForAttribute,
    AS: valid::Attributes,
{ }
