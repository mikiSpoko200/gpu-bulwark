//! This module provides specialization of HLists for Vertex Array Object attributes.

use crate::gl;
use crate::gl::buffer;
use crate::constraint;
use crate::ffi;
use crate::glsl;
use crate::valid;
use crate::md;
use crate::types;
use crate::hlist;

use crate::prelude::internal::*;

pub(crate) struct Attribute<'buffer, T, > {
    pub buffer: &'buffer buffer::Buffer<buffer::Array, A>,
}

impl<T> valid::ForAttribute for T where T: valid::ForBuffer<buffer::Array> { }

impl<'buffer, A, AS, const INDEX: usize> valid::Attributes for (AS, Attribute<'buffer, A, INDEX>)
where
    A: valid::ForAttribute,
    AS: valid::Attributes,
{ }
