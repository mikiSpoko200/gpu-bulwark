/// Provide binding point

use crate::prelude::internal::*;

use crate::gl;

use gl::buffer::{Buffer, Target};

pub struct VertexBufferBinding<'buffer, const BINDING_INDEX: usize, T>(&'buffer Buffer<T>)
where
    T: Target;
