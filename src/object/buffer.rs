use super::{prelude::*, resource};
use super::resource::{Bindable, Allocator};
use crate::prelude::Const;
use crate::target::{buffer, buffer::format};
use crate::{error, gl_call};
use gl::types::{GLenum, GLuint};
use std::marker::PhantomData;

/// Type level enumeration of possible Buffer data Usage types
pub trait Usage: Const<GLenum> {}

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
#[derive(Default)]
struct BufferSemantics<T, F>
where
    T: buffer::Target,
    (T, F): format::Valid
{
    _target_phantom: PhantomData<T>,
    _format_phantom: PhantomData<F>,
}

/// Allocation strategy for OpenGL buffer objects.
struct BufferAllocator;

unsafe impl resource::Allocator for BufferAllocator {
    fn allocate(names: &mut [Name]) {
        unsafe {
            gl::CreateBuffers(names.len() as _, names as _);
        }
    }

    fn free(names: &[Name]) {
        unsafe {
            gl::DeleteBuffers(names.len() as _, names as _);
        }
    }
}

type BufferObject = Object<BufferAllocator>;

pub struct Buffer<T, F>
where
    T: buffer::Target,
    (T, F): format::Valid,
{
    object: BufferObject,
    _semantic: BufferSemantics<T, F>
}

impl<T, F> Buffer<T, F>
where
    T: buffer::Target,
    (T, F): format::Valid,
{
    pub fn data<U>(&self, data: &[F])
    where
        U: Usage,
    {
        // TODO: error handling
        self.bind();
        gl_call! { #[panic] unsafe {
                gl::BufferData(
                    T::BIND_TARGET,
                    data.len() as _,
                    data.as_ptr() as _,
                    U::VALUE,
                );
            }
        }
        self.unbind();
    }
}

impl<T, F> Bindable for Buffer<T, F>
where
    T: buffer::Target,
    (T, F): format::Valid,
{
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe { gl::BindBuffer(T::BIND_TARGET, self.object.name()) }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe { gl::BindBuffer(T::BIND_TARGET, 0) }
        }
    }
}
