use crate::constraint;
use crate::glsl;
use crate::glsl::marker::Scalar;
use crate::glsl::Const;
use crate::impl_target;
use crate::mode;

use hi;

use glsl::marker::Vector;

/// Buffer object target types.
pub(crate) unsafe trait Target: crate::target::Target {}

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct Array;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct AtomicCounter;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct CopyRead;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct CopyWrite;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct DispatchIndirect;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct DrawIndirect;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct ElementArray;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct PixelPack;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct PixelUnpack;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct Query;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct ShaderStorage;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct Texture;

#[derive(Default)]
#[hi::mark(mode::Validation)]
pub struct TransformFeedback; 

#[derive(Default)]
#[hi::mark(mode::Validation)]
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


impl<T, const SIZE: usize> constraint::Valid<Array> for glsl::base::Vec<T, SIZE>
where
    T: glsl::Type + constraint::Valid<Vector>,
    Const<SIZE>: constraint::Valid<Vector>,
{
}
