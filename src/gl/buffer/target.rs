use crate::constraint;
use crate::glsl;
use crate::impl_target;
use crate::md;
use crate::gl;

/// Buffer object target types.
#[hi::marker]
pub(crate) trait Target: gl::target::Target { }

#[hi::mark(Target)]
pub enum Array { }

#[hi::mark(Target)]
pub enum AtomicCounter { }

#[hi::mark(Target)]
pub enum CopyRead { }

#[hi::mark(Target)]
pub enum CopyWrite { }

#[hi::mark(Target)]
pub enum DispatchIndirect { }

#[hi::mark(Target)]
pub enum DrawIndirect { }

#[hi::mark(Target)]
pub enum ElementArray { }

#[hi::mark(Target)]
pub enum PixelPack { }

#[hi::mark(Target)]
pub enum PixelUnpack { }

#[hi::mark(Target)]
pub enum Query { }

#[hi::mark(Target)]
pub enum ShaderStorage { }

#[hi::mark(Target)]
pub enum Texture { }

#[hi::mark(Target)]
pub enum TransformFeedback { } 

#[hi::mark(Target)]
pub enum Uniform { }

impl_target!{ Array as ARRAY_BUFFER }
impl_target!{ AtomicCounter as ATOMIC_COUNTER_BUFFER }
impl_target!{ CopyRead as COPY_READ_BUFFER }
impl_target!{ CopyWrite as COPY_WRITE_BUFFER }
impl_target!{ DispatchIndirect as DISPATCH_INDIRECT_BUFFER }
impl_target!{ DrawIndirect as DRAW_INDIRECT_BUFFER }
impl_target!{ ElementArray as ELEMENT_ARRAY_BUFFER }
impl_target!{ PixelPack as PIXEL_PACK_BUFFER }
impl_target!{ PixelUnpack as PIXEL_UNPACK_BUFFER }
impl_target!{ Query as QUERY_BUFFER }
impl_target!{ ShaderStorage as SHADER_STORAGE_BUFFER }
impl_target!{ Texture as TEXTURE_BUFFER }
impl_target!{ TransformFeedback as TRANSFORM_FEEDBACK_BUFFER }
impl_target!{ Uniform as UNIFORM_BUFFER }
