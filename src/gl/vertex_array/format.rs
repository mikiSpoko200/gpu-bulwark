use crate::prelude::internal::*;

use crate::gl;
use gl::vertex_array::bounds::AttribFormat;

pub struct Format<const ATTRIB_INDEX: usize, T: AttribFormat, const RELATIVE_OFFSET: u16 = 0>(PhantomData<T>);
