//! TODO: FILL ME!

use crate::glsl::InVariable;
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

use super::bounds::AttribFormat;

#[derive(dm::AsRef)]
pub struct Attribute<GL, const ATTRIB_INDEX: usize>
where
    GL: bounds::AttribFormat,
{
    #[as_ref(forward)]
    var: VertexBufferBinding<GL>,
    format: Format<ATTRIB_INDEX, GL>
}

impl<GL, const ATTRIB_INDEX: usize> Attribute<GL, ATTRIB_INDEX>
where
    GL: bounds::AttribFormat,
{
    pub(in crate::gl) fn new(vbo: buffer::Buffer<target::Array, GL>) -> Self {
        Self {
            format: Format::new(&vbo),
            var: VertexBufferBinding::new(vbo),
        }
    }
}
