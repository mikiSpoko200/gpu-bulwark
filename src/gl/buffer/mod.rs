//! OpenGL Buffer Object.
//! 
//! ## Design
//! 
//! Buffer objects serve fundamentally different purpose then program or shader as they express configuration of the pipeline which can be entirely expressed using glsl types.
//! Buffers on the other hand are mainly used by the OpenGL API and are never explicitly named by in shader code.
//! 
//! This distinction manifests itself in API calls like `VertexAttribPointer`
//! 
//! In case for vertices, a VAO is a bridge between raw data from Buffer and vertex data specification in shader code.
//! Let's be faithful to that
//! 

pub mod target;
pub mod _valid;


use super::object;
use crate::gl;
use gl::buffer;
use crate::utils::Const;
use crate::types::Primitive;
use crate::error;
use crate::glsl;
use crate::valid;
use glb::types::{GLenum, GLuint};
use object::*;

pub use target::*;

use crate::prelude::internal::*;

/// Type level enumeration of possible Buffer data Usage types
pub trait Usage {
    const ID: u32;
}

macro_rules! impl_usage {
    ($ty:ty: $id:expr ) => {
        impl Usage for $ty {
            const ID: u32 = $id;
        }
    };
}

pub enum Stream { }

pub enum Static { }

pub enum Dynamic { }

pub enum Draw { }

pub enum Read { }

pub enum Copy { }

// impls for Stream access frequency
impl_usage!((Stream, Draw): glb::STREAM_DRAW);
impl_usage!((Stream, Read): glb::STREAM_READ);
impl_usage!((Stream, Copy): glb::STREAM_COPY);

// impls for Static access frequency
impl_usage!((Static, Draw): glb::STATIC_DRAW);
impl_usage!((Static, Read): glb::STATIC_READ);
impl_usage!((Static, Copy): glb::STATIC_COPY);

// impls for Dynamic access frequency
impl_usage!((Dynamic, Draw): glb::DYNAMIC_DRAW);
impl_usage!((Dynamic, Read): glb::DYNAMIC_READ);
impl_usage!((Dynamic, Copy): glb::DYNAMIC_COPY);


/// Allocator for OpenGL buffer objects.
#[hi::mark(PartialObject, Object)]
struct BufferObject<T: Target>(PhantomData<T>);

unsafe impl<T: Target> object::Allocator for BufferObject<T> {
    fn allocate(names: &mut [u32]) {
        unsafe {
            glb::CreateBuffers(names.len() as _, names.as_mut_ptr());
        }
    }

    fn free(names: &[u32]) {
        unsafe {
            glb::DeleteBuffers(names.len() as _, names.as_ptr());
        }
    }
}

impl<T: Target> object::Binder for BufferObject<T> {
    fn bind(name: u32) {
        gl::call! {
            [panic]
            unsafe { glb::BindBuffer(T::ID, name) }
        }
    }
}

pub(crate) struct BufferState<T, F>
where
    T: buffer::Target,
{
    _phantoms: PhantomData<(T, F)>,
    pub(crate) length: usize,
}

impl<T, F> Default for BufferState<T, F>
where
    T: buffer::Target,
{
    fn default() -> Self {
        Self {
            _phantoms: PhantomData,
            length: 0,
        }
    }
}

#[derive(dm::Deref)]
pub struct Buffer<T, GL>
where
    T: buffer::Target,
{
    #[deref]
    object: ObjectBase<BufferObject<T>>,
    pub(crate) state: BufferState<T, GL>,
}

impl<T, GL> Default for Buffer<T, GL>
where
    T: buffer::Target,
{
    fn default() -> Self {
        Self {
            object: Default::default(),
            state: Default::default(),
        }
    }
}

impl<T, GL> Buffer<T, GL>
where
    T: buffer::Target,
{
    pub fn create() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn data<U>(&mut self, data: &[GL])
    where
        U: Usage,
    {
        {
            gl::call! {
                [panic]
                unsafe {
                    glb::BufferData(
                        T::ID,
                        (std::mem::size_of::<GL>() * data.len()) as _,
                        data.as_ptr() as _,
                        U::ID,
                    );
                }
            }
            self.state.length = data.len();
        };
    }
}
