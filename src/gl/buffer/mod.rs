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
pub trait Usage: Const<u32> { }

pub enum Stream { }

pub enum Static { }

pub enum Dynamic { }

pub enum Draw { }

pub enum Read { }

pub enum Copy { }

// impls for Stream access frequency
crate::impl_const_super_trait!(Usage for (Stream, Draw), glb::STREAM_DRAW);
crate::impl_const_super_trait!(Usage for (Stream, Read), glb::STREAM_READ);
crate::impl_const_super_trait!(Usage for (Stream, Copy), glb::STREAM_COPY);

// impls for Static access frequency
crate::impl_const_super_trait!(Usage for (Static, Draw), glb::STATIC_DRAW);
crate::impl_const_super_trait!(Usage for (Static, Read), glb::STATIC_READ);
crate::impl_const_super_trait!(Usage for (Static, Copy), glb::STATIC_COPY);

// impls for Dynamic access frequency
crate::impl_const_super_trait!(Usage for (Dynamic, Draw), glb::DYNAMIC_DRAW);
crate::impl_const_super_trait!(Usage for (Dynamic, Read), glb::DYNAMIC_READ);
crate::impl_const_super_trait!(Usage for (Dynamic, Copy), glb::DYNAMIC_COPY);


/// Allocator for OpenGL buffer objects.
enum BufferAllocator { }

unsafe impl object::Allocator for BufferAllocator {
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

struct BufferBinder<T: Target>(PhantomData<T>);

impl<T: Target> object::Binder for BufferBinder<T> {
    fn bind(name: u32) {
        gl::call! {
            [panic]
            unsafe { glb::BindBuffer(T::VALUE, name) }
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
    F: valid::ForBuffer<T>,
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
    GL: valid::ForBuffer<T>,
{
    #[deref]
    object: ObjectBase<Self>,
    pub(crate) state: BufferState<T, GL>,
}

impl<T, GLSL> Default for Buffer<T, GLSL>
where
    T: buffer::Target,
    GLSL: valid::ForBuffer<T>,
{
    fn default() -> Self {
        Self {
            object: Default::default(),
            state: Default::default(),
        }
    }
}

impl<T, GL> Object for Buffer<T, GL>
where
    T: buffer::Target,
    GL: valid::ForBuffer<T>,
{
    type Binder = BufferBinder<T>;
    type Allocator = BufferAllocator;
}

impl<T, GLSL> Buffer<T, GLSL>
where
    T: buffer::Target,
    GLSL: valid::ForBuffer<T>,
{
    pub fn create() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn data<U, GL>(&mut self, data: &[GL])
    where
        U: Usage,
        GL: glsl::Compatible<GLSL>,
    {
        {
            gl::call! {
                [panic]
                unsafe {
                    glb::BufferData(
                        T::VALUE,
                        (std::mem::size_of::<GL::Layout>() * data.len()) as _,
                        data.as_ptr() as _,
                        U::VALUE,
                    );
                }
            }
            self.state.length = data.len();
        };
    }
}
