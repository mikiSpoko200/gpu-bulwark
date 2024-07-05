use crate::constraint;
use crate::glsl;
use crate::impl_target;
use crate::mode;
use crate::gl;

/// Buffer object target types.
#[hi::marker]
pub(crate) trait Target: gl::target::Target { }

#[macro_export]
#[allow(unused)]
macro_rules! impl_target {
    ($target_type:ty as $gl_target_ident: ident) => {
        impl $crate::gl::target::Target for $target_type {
            const VALUE: u32 = glb::$gl_target_ident;
        }
        impl $crate::gl::buffer::target::Target for $target_type { }
    };
}

#[hi::mark(mode::Validation)]
pub enum Array { }

#[hi::mark(mode::Validation)]
pub enum AtomicCounter { }

#[hi::mark(mode::Validation)]
pub enum CopyRead { }

#[hi::mark(mode::Validation)]
pub enum CopyWrite { }

#[hi::mark(mode::Validation)]
pub enum DispatchIndirect { }

#[hi::mark(mode::Validation)]
pub enum DrawIndirect { }

#[hi::mark(mode::Validation)]
pub enum ElementArray { }

#[hi::mark(mode::Validation)]
pub enum PixelPack { }

#[hi::mark(mode::Validation)]
pub enum PixelUnpack { }

#[hi::mark(mode::Validation)]
pub enum Query { }

#[hi::mark(mode::Validation)]
pub enum ShaderStorage { }

#[hi::mark(mode::Validation)]
pub enum Texture { }

#[hi::mark(mode::Validation)]
pub enum TransformFeedback { } 

#[hi::mark(mode::Validation)]
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
