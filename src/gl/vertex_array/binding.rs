/// Provide binding point

use crate::prelude::internal::*;

use crate::gl;

use gl::object::ObjectBase;
use gl::buffer::{Buffer, target};

#[derive(dm::AsRef)]
pub struct VertexBufferBinding<const BINDING_INDEX: usize, GL>(pub(in crate::gl::vertex_array) Buffer<target::Array, GL>);

impl<const BINDING_INDEX: usize, GL> VertexBufferBinding<BINDING_INDEX,  GL> {
    pub(in crate::gl) fn new(vbo: Buffer<target::Array, GL>) -> Self {
        Self(vbo)
    }
}
