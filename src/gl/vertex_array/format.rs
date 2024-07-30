use crate::prelude::internal::*;

use crate::gl;
use gl::buffer::target;
use gl::vertex_array::bounds::AttribFormat;

pub struct Format<const ATTRIB_INDEX: usize, T: AttribFormat, const RELATIVE_OFFSET: u16 = 0>(PhantomData<T>);


impl<const ATTRIB_INDEX: usize, T: AttribFormat, const RELATIVE_OFFSET: u16> Format<ATTRIB_INDEX, T, RELATIVE_OFFSET> {
    pub (in crate::gl) fn new(vbo: &gl::Buffer<target::Array, T>) -> Self {
        Self(PhantomData)
    }
}
