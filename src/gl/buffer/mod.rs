pub mod target;

use super::resource::{Allocator, Bind};
use super::{prelude::*, resource};
use crate::utils::Const;
use crate::types::Primitive;
use crate::{constraint, error, gl_call, glsl, mode};
use glb::types::{GLenum, GLuint};
use std::marker::PhantomData;
use target as buffer;

/// Type level enumeration of possible Buffer data Usage types
pub trait Usage: Const<u32> {}

pub struct Stream;

pub struct Static;

pub struct Dynamic;

pub struct Draw;

pub struct Read;

pub struct Copy;

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

/// Use to enforce semantics for OpenGL buffer object.
pub(crate) struct BufferSemantics<T, F>
where
    T: buffer::Target + mode::Validation,
    F: constraint::Valid<T>,
{
    _phantoms: PhantomData<(T, F)>,
    pub(crate) length: usize,
}

impl<T, F> Default for BufferSemantics<T, F>
where
    T: buffer::Target + mode::Validation,
    F: constraint::Valid<T>,
{
    fn default() -> Self {
        Self {
            _phantoms: PhantomData,
            length: 0,
        }
    }
}

/// Allocation strategy for OpenGL buffer objects.
struct BufferAllocator;

unsafe impl resource::Allocator for BufferAllocator {
    fn allocate(names: &mut [Name]) {
        unsafe {
            glb::CreateBuffers(names.len() as _, names.as_mut_ptr());
        }
    }

    fn free(names: &[Name]) {
        unsafe {
            glb::DeleteBuffers(names.len() as _, names.as_ptr());
        }
    }
}

type BufferObject = Object<BufferAllocator>;

pub struct Buffer<T, GLSL>
where
    T: buffer::Target + mode::Validation,
    GLSL: constraint::Valid<T>,
{
    object: BufferObject,
    pub(crate) semantics: BufferSemantics<T, GLSL>,
}

impl<T, F> Default for Buffer<T, F>
where
    T: buffer::Target + mode::Validation,
    F: constraint::Valid<T>,
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
    T: buffer::Target + mode::Validation,
    GLSL: glsl::Type<Group = glsl::marker::Transparent> + constraint::Valid<T>,
{
    pub fn create() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn data<U, GL>(&mut self, data: &[GL])
    where
        U: Usage,
        GL: glsl::Compatible<GLSL, Layout = GLSL::Layout>,
    {
        self.bind();
        gl_call! {
            #[panic]
            unsafe {
                glb::BufferData(
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
    T: buffer::Target + mode::Validation,
    F: glsl::Type + constraint::Valid<T>,
{
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe { glb::BindBuffer(T::VALUE, self.object.name()) }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe { glb::BindBuffer(T::VALUE, 0) }
        }
    }
}
