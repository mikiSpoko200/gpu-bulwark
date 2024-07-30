/// Provide binding point

use crate::prelude::internal::*;

use crate::gl;

use gl::object::ObjectBase;
use gl::buffer::{Buffer, BufferObject, Target, target};

pub struct VertexBufferBinding<'buffer, const BINDING_INDEX: usize>(&'buffer ObjectBase<BufferObject<target::Array>>);

impl<'buffer, const BINDING_INDEX: usize> VertexBufferBinding<'buffer, BINDING_INDEX> {
    pub(in crate::gl) fn new<GL>(vbo: &'buffer Buffer<target::Array, GL>) -> Self {
        Self(vbo)
    }
}
