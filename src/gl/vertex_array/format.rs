use crate::prelude::internal::*;

use crate::gl;
use gl::buffer::target;
use gl::vertex_array::bounds::AttribFormat;

pub struct Format<const ATTRIB_INDEX: usize, T: AttribFormat> {
    phantom: PhantomData<T>,
    relative_offset: usize,
}

impl<const ATTRIB_INDEX: usize, T: AttribFormat> Format<ATTRIB_INDEX, T> {
    pub fn with_relative_offset(_: &gl::Buffer<target::Array, T>, relative_offset: usize) -> Self {
        Self {
            phantom: PhantomData,
            relative_offset,
        }
    }
}

impl<const ATTRIB_INDEX: usize, T: AttribFormat> Format<ATTRIB_INDEX, T> {
    pub (in crate::gl) fn new(buffer: &gl::Buffer<target::Array, T>) -> Self {
        Self::with_relative_offset(buffer, 0)
    }
}
