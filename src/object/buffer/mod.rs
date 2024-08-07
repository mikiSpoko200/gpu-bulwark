pub mod target;

use super::resource::{Allocator, Bind};
use super::{prelude::*, resource};
use crate::prelude::Const;
use crate::types::Primitive;
use target as buffer;
use crate::{error, gl_call, glsl};
use gl::types::{GLenum, GLuint};
use std::marker::PhantomData;

/// Type level enumeration of possible Buffer data Usage types
pub trait Usage: Const<u32> {}

pub struct Stream;

pub struct Static;

pub struct Dynamic;

pub struct Draw;

pub struct Read;

pub struct Copy;

// impls for Stream access frequency
crate::impl_const_super_trait!(Usage for (Stream, Draw), gl::STREAM_DRAW);
crate::impl_const_super_trait!(Usage for (Stream, Read), gl::STREAM_READ);
crate::impl_const_super_trait!(Usage for (Stream, Copy), gl::STREAM_COPY);

// impls for Static access frequency
crate::impl_const_super_trait!(Usage for (Static, Draw), gl::STATIC_DRAW);
crate::impl_const_super_trait!(Usage for (Static, Read), gl::STATIC_READ);
crate::impl_const_super_trait!(Usage for (Static, Copy), gl::STATIC_COPY);

// impls for Dynamic access frequency
crate::impl_const_super_trait!(Usage for (Dynamic, Draw), gl::DYNAMIC_DRAW);
crate::impl_const_super_trait!(Usage for (Dynamic, Read), gl::DYNAMIC_READ);
crate::impl_const_super_trait!(Usage for (Dynamic, Copy), gl::DYNAMIC_COPY);

/// Use to enforce semantics for OpenGL buffer object.
pub(crate) struct BufferSemantics<T, F>
where
    T: buffer::Target,
    F: buffer::format::Valid<T>,
{
    _target_phantom: PhantomData<T>,
    _format_phantom: PhantomData<F>,
    pub(crate) length: usize,
}

impl<T, F> Default for BufferSemantics<T, F>
where
    T: buffer::Target,
    F: buffer::format::Valid<T>,
{
    fn default() -> Self {
        Self {
            _target_phantom: PhantomData,
            _format_phantom: PhantomData,
            length: 0,
        }
    }
}

/// Allocation strategy for OpenGL buffer objects.
struct BufferAllocator;

unsafe impl resource::Allocator for BufferAllocator {
    fn allocate(names: &mut [Name]) {
        unsafe {
            gl::CreateBuffers(names.len() as _, names.as_mut_ptr());
        }
    }

    fn free(names: &[Name]) {
        unsafe {
            gl::DeleteBuffers(names.len() as _, names.as_ptr());
        }
    }
}

type BufferObject = Object<BufferAllocator>;

pub struct Buffer<T, GLSL>
where
    T: buffer::Target,
    GLSL: buffer::format::Valid<T>,
{
    object: BufferObject,
    pub(crate) semantics: BufferSemantics<T, GLSL>,
}

impl<T, F> Default for Buffer<T, F>
where
    T: buffer::Target,
    F: buffer::format::Valid<T>,
{
    fn default() -> Self {
        Self {
            object: Default::default(),
            semantics: Default::default(),
        }
    }
}

impl<T, GLSL> Buffer<T, GLSL>
where
    T: buffer::Target,
    GLSL: buffer::format::Valid<T>,
{
    pub fn create() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn data<U, GL>(&mut self, data: &[GL])
    where
        U: Usage,
        GL: glsl::compatible::Compatible<GLSL, Primitive = GLSL::Primitive>
    {
        // TODO: error handling
        self.bind();
        gl_call! { 
            #[panic]
            unsafe {
                gl::BufferData(
                    T::VALUE,
                    (std::mem::size_of::<GLSL::Primitive>() * data.len()) as _,
                    data.as_ptr() as _,
                    U::VALUE,
                );
            }
        }
        self.semantics.length = data.len();
        self.unbind();
    }
}

impl<T, F> Bind for Buffer<T, F>
where
    T: buffer::Target,
    F: buffer::format::Valid<T>,
{
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe { gl::BindBuffer(T::VALUE, self.object.name()) }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe { gl::BindBuffer(T::VALUE, 0) }
        }
    }
}
