use super::prelude::*;
use super::resource::{Bindable, Resource};
use crate::prelude::Const;
use crate::targets::buffer;
use crate::{error, gl_call};
use gl::types::{GLenum, GLuint};
use std::marker::PhantomData;

pub struct Buffer<Target>
where
    Target: buffer::Target,
{
    base: Object,
    _target_phantom: PhantomData<Target>,
}

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

impl<Target> Buffer<Target>
where
    Target: buffer::Target,
{
    fn data<U>(&self, data: &[u8])
    where
        U: Usage,
    {
        // TODO: error handling
        gl_call! { #[panic] unsafe {
                gl::BufferData(
                    Target::BIND_TARGET,
                    data.len() as _,
                    data.as_ptr() as _,
                    U::VALUE,
                );
            }
        }
    }
}

impl<Target> From<Object> for Buffer<Target>
where
    Target: buffer::Target,
{
    fn from(base: Object) -> Self {
        Self {
            base,
            _target_phantom: Default::default(),
        }
    }
}

impl<Target> Into<Object> for Buffer<Target>
where
    Target: buffer::Target,
{
    fn into(self) -> Object {
        let Self { base, .. } = self;
        base
    }
}

impl<Target> Bindable for Buffer<Target>
where
    Target: buffer::Target,
{
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe { gl::BindBuffer(Target::BIND_TARGET, self.base.0) }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe { gl::BindBuffer(Target::BIND_TARGET, 0) }
        }
    }
}

impl<Target> Resource for Buffer<Target>
where
    Target: buffer::Target,
{
    type Ok = ();

    fn initialize(names: &mut [GLuint]) -> error::Result<Self::Ok> {
        gl_call! {
            #[propagate]
            unsafe { gl::CreateBuffers(names.len() as _, names.as_mut_ptr()); }
        }
    }

    fn free(names: &[GLuint]) -> error::Result<Self::Ok> {
        gl_call! {
            #[propagate]
            unsafe { gl::DeleteBuffers(names.len() as _, names.as_ptr()); }
        }
    }
}
