use crate::impl_target;

use gl::types::GLenum;

/// A target marker for a buffer object
pub unsafe trait Target {
    const BIND_TARGET: GLenum;
}

pub struct Array;
pub struct AtomicCounter;
pub struct CopyRead;
pub struct CopyWrite;
pub struct DispatchIndirect;
pub struct DrawIndirect;
pub struct ElementArray;
pub struct PixelPack;
pub struct PixelUnpack;
pub struct Query;
pub struct ShaderStorage;
pub struct Texture;
pub struct TransformFeedback;
pub struct Uniform;

impl_target!(buffer, Array, ARRAY_BUFFER);
impl_target!(buffer, AtomicCounter, ATOMIC_COUNTER_BUFFER);
impl_target!(buffer, CopyRead, COPY_READ_BUFFER);
impl_target!(buffer, CopyWrite, COPY_WRITE_BUFFER);
impl_target!(buffer, DispatchIndirect, DISPATCH_INDIRECT_BUFFER);
impl_target!(buffer, DrawIndirect, DRAW_INDIRECT_BUFFER);
impl_target!(buffer, ElementArray, ELEMENT_ARRAY_BUFFER);
impl_target!(buffer, PixelPack, PIXEL_PACK_BUFFER);
impl_target!(buffer, PixelUnpack, PIXEL_UNPACK_BUFFER);
impl_target!(buffer, Query, QUERY_BUFFER);
impl_target!(buffer, ShaderStorage, SHADER_STORAGE_BUFFER);
impl_target!(buffer, Texture, TEXTURE_BUFFER);
impl_target!(buffer, TransformFeedback, TRANSFORM_FEEDBACK_BUFFER);
impl_target!(buffer, Uniform, UNIFORM_BUFFER);

pub mod format {
    /// Relation of types being valid data formats for given target.
    pub unsafe trait Valid { }

    /// This exploits the 3rd rule of unconstrained type parameter exceptions
    /// "be bound as an associated type."
    pub trait Id {
        type Id;
    }

    // Doing this through a blanket impl would yield two main benefits:
    // 1. DRY - relation between Gl/Glsl would be established only once and rest would follow.
    // 2. I forgot the second one.

    unsafe impl Valid for (super::Array, [f32; 3]) { }
    unsafe impl Valid for (super::Array, [f32; 4]) { }
    unsafe impl Valid for (super::Array, f32) { }
}
