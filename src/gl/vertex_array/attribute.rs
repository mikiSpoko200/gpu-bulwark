//! TODO: FILL ME!

use crate::prelude::internal::*;

use crate::gl;
use crate::gl::buffer;
use crate::constraint;
use crate::ffi;
use crate::glsl;
use crate::valid;
use crate::md;
use crate::hlist;

use buffer::target;
use gl::types;
use gl::vertex_array::bounds;
use gl::vertex_array::{Format, VertexBufferBinding};

pub struct Attribute<'buffer, GL, const ATTRIB_INDEX: usize, const BINDING_INDEX: usize = ATTRIB_INDEX>
where
    GL: bounds::AttribFormat,
{
    binding: VertexBufferBinding<'buffer, BINDING_INDEX>,
    format: Format<ATTRIB_INDEX, GL>
}

impl<'buffer, GL, const ATTRIB_INDEX: usize, const BINDING_INDEX: usize> Attribute<'buffer, GL, ATTRIB_INDEX, BINDING_INDEX>
where
    GL: bounds::AttribFormat,
{
    pub(in crate::gl) fn new(vbo: &'buffer buffer::Buffer<target::Array, GL>) -> Self {
        Self {
            binding: VertexBufferBinding::new(vbo),
            format: Format::new(vbo),
        }
    }
}
