use super::prelude::*;
use super::resource::{Bindable, Handle, Resource};
use crate::prelude::Const;
use crate::targets::{buffer, buffer::format};
use crate::{error, gl_call};
use gl::types::{GLenum, GLuint};
use std::marker::PhantomData;

pub struct Buffer<Target, Data>
where
    Target: buffer::Target,
    (Target, Data): format::Valid,
{
    base: Object,
    _target_phantom: PhantomData<Target>,
    _format_phantom: PhantomData<Data>,
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

impl<Target, Data> Buffer<Target, Data>
where
    Target: buffer::Target,
    (Target, Data): format::Valid,
{
    pub fn data<U>(&self, data: &[Data])
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

impl<Target, Data> From<Object> for Buffer<Target, Data>
where
    Target: buffer::Target,
    (Target, Data): format::Valid,
{
    fn from(base: Object) -> Self {
        Self {
            base,
            _target_phantom: Default::default(),
            _format_phantom: Default::default(),
        }
    }
}

impl<Target, Data> Into<Object> for Buffer<Target, Data>
where
    Target: buffer::Target,
    (Target, Data): format::Valid,
{
    fn into(self) -> Object {
        let Self { base, .. } = self;
        base
    }
}

impl<Target, Data> Bindable for Buffer<Target, Data>
where
    Target: buffer::Target,
    (Target, Data): format::Valid,
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

impl<Target, Data> Resource for Buffer<Target, Data>
where
    Target: buffer::Target,
    (Target, Data): format::Valid,
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

pub fn make<Data>() -> Handle<Buffer<buffer::Array, Data>>
where
    (buffer::Array, Data): format::Valid,
{
    Handle::new()
}
