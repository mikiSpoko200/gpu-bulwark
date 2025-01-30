use crate::constraint;
use crate::glsl;
use crate::impl_target;
use crate::md;
use crate::gl;
use gl::texture;

/// Buffer object target types.
#[hi::marker]
pub trait Target: gl::target::Target { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum Array { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum AtomicCounter { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum CopyRead { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum CopyWrite { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum DispatchIndirect { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum DrawIndirect { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum ElementArray { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum PixelPack { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum PixelUnpack { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum Query { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum ShaderStorage { }

#[hi::mark(Target)]
#[derive(Debug)]
pub enum TransformFeedback { } 

#[hi::mark(Target)]
#[derive(Debug)]
pub enum Uniform { }

hi::denmark! { texture::Buffer as Target }

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
impl_target!{ TransformFeedback as TRANSFORM_FEEDBACK_BUFFER }
impl_target!{ Uniform as UNIFORM_BUFFER }
