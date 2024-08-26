/// Provide var point

use crate::prelude::internal::*;

use crate::gl;

use gl::object::ObjectBase;
use gl::buffer::{Buffer, target};

// NOTE: Single buffer in the future should provide bindings due to containing multiple type arrays consecutively in memory. Bindings would be set per one such type.
// This means that Binding does not necessarily contains a buffer, but rather reference to single type of type list in buffer.
// All of this is temporarily ignored and VertexBufferBinding contains a whole buffer to preserve structure.

#[derive(dm::AsRef, dm::AsMut)]
pub struct VertexBufferBinding<GL>(pub(in crate::gl) Buffer<target::Array, GL>);

impl<GL> VertexBufferBinding<GL> {
    pub(in crate::gl) fn new(vbo: Buffer<target::Array, GL>) -> Self {
        Self(vbo)
    }
}
