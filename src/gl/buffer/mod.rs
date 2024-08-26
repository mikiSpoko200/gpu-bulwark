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


use std::os::raw::c_void;

use super::object;
use crate::gl;
use crate::utils::Const;
use gl::buffer;
use gl::error;
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
pub struct BufferObject<T: Target>(PhantomData<T>);

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

pub(crate) struct BufferState<F> {
    _phantoms: PhantomData<F>,
    pub(crate) length: usize,
}

impl<F> Default for BufferState<F> {
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
    pub(in crate::gl) state: BufferState<GL>,
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
            if self.state.length > 0 && self.state.length != data.len() {
                panic!("realocating buffers with mutable storage is not supported");
            }
            let binder = self.bind();
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

    pub fn len(&self) -> usize {
        self.state.length
    }

    pub fn map(&self) -> impl std::ops::Deref<Target=&[GL]> {
        MappedRef::new(self)
    }

    pub fn map_mut(&mut self) -> impl std::ops::DerefMut<Target = &mut [GL]> {
        MappedMut::new(self)
    }
}


#[derive(dm::Deref)]
pub struct MappedRef<'b, T, GL>(&'b Buffer<T, GL>, #[deref] &'b [GL])
where
    T: buffer::Target,
;

impl<'b, T, GL> MappedRef<'b, T, GL>
where
    T: buffer::Target,
{
    pub(super) fn new(buffer: &'b Buffer<T, GL>) -> Self {
        let binding = buffer.bind();
        let mut data;
        gl::call! {
            [panic]
            unsafe {
                data = glb::MapBuffer(T::ID, glb::READ_ONLY) as *const _;
            }
        }
        // SAFETY: [spec] if no error was generated pointer is valid
        let slice = unsafe { std::slice::from_raw_parts(data, buffer.len()) };
        Self(buffer, slice)
    }
}

impl<'b, T, GL> Drop for MappedRef<'b, T, GL>
where
    T: buffer::Target,
{   
    fn drop(&mut self) {
        let binding = self.0.bind();
        gl::call! {
            [panic]
            unsafe {
                glb::UnmapBuffer(T::ID);
            }
        }
    }
}



#[derive(dm::Deref, dm::DerefMut)]
pub struct MappedMut<'b, T, GL>(&'b mut Buffer<T, GL>, #[deref] #[deref_mut] &'b mut [GL])
where
    T: buffer::Target,
;

impl<'b, T, GL> MappedMut<'b, T, GL>
where
    T: buffer::Target,
{
    pub(super) fn new(buffer: &'b mut Buffer<T, GL>) -> Self {
        let binding = buffer.bind();
        let mut data;
        gl::call! {
            [panic]
            unsafe {
                data = glb::MapBuffer(T::ID, glb::READ_WRITE) as *mut _;
            }
        }
        // SAFETY: [spec] if no error was generated pointer is valid
        let slice = unsafe { 
            std::slice::from_raw_parts_mut(data, buffer.len())
        };
        Self(buffer, slice)
    }
}

impl<'b, T, GL> Drop for MappedMut<'b, T, GL>
where
    T: buffer::Target,
{   
    fn drop(&mut self) {
        let binding = self.0.bind();
        gl::call! {
            [panic]
            unsafe {
                glb::UnmapBuffer(T::ID);
            }
        }
    }
}
