use std::marker::PhantomData;
use gl::types::{GLenum, GLuint};
use crate::error;
use crate::prelude::Const;
use super::prelude::*;
use crate::targets::buffer;
use super::resource::{Resource, Bindable};

pub struct Buffer<Target> where Target: buffer::Target {
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
impl Const<GLenum> for (Stream, Draw) { const VALUE: GLenum = gl::STREAM_DRAW; }

impl Usage for (Stream, Draw) {}

impl Const<GLenum> for (Stream, Read) { const VALUE: GLenum = gl::STREAM_READ; }

impl Usage for (Stream, Read) {}

impl Const<GLenum> for (Stream, Copy) { const VALUE: GLenum = gl::STREAM_COPY; }

impl Usage for (Stream, Copy) {}


// impls for Static access frequency
impl Const<GLenum> for (Static, Draw) { const VALUE: GLenum = gl::STATIC_DRAW; }

impl Usage for (Static, Draw) {}

impl Const<GLenum> for (Static, Read) { const VALUE: GLenum = gl::STATIC_READ; }

impl Usage for (Static, Read) {}

impl Const<GLenum> for (Static, Copy) { const VALUE: GLenum = gl::STATIC_COPY; }

impl Usage for (Static, Copy) {}

// impls for Dynamic access frequency
impl Const<GLenum> for (Dynamic, Draw) { const VALUE: GLenum = gl::DYNAMIC_DRAW; }

impl Usage for (Dynamic, Draw) {}

impl Const<GLenum> for (Dynamic, Read) { const VALUE: GLenum = gl::DYNAMIC_READ; }

impl Usage for (Dynamic, Read) {}

impl Const<GLenum> for (Dynamic, Copy) { const VALUE: GLenum = gl::DYNAMIC_COPY; }

impl Usage for (Dynamic, Copy) {}

impl<Target> Buffer<Target> where Target: buffer::Target {
    fn data<U>(&self, data: &[u8])
        where
            U: Usage
    {
        // TODO: error handling
        unsafe {
            gl::BufferData(
                Target::BIND_TARGET,
                data.len() as _,
                data.as_ptr() as _,
                U::VALUE
            );
        }
    }
}

impl<Target> From<Object> for Buffer<Target> where Target: buffer::Target {
    fn from(base: Object) -> Self {
        Self { base, _target_phantom: Default::default() }
    }
}

impl<Target> Into<Object> for Buffer<Target> where Target: buffer::Target {
    fn into(self) -> Object {
        let Self { base, .. } = self;
        base
    }
}

impl<Target> Bindable for Buffer<Target> where Target: buffer::Target {
    fn bind(&self) {
        unsafe {
            gl::BindBuffer(Target::BIND_TARGET, self.base.0)
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::BindBuffer(Target::BIND_TARGET, 0);
        }
    }
}

impl<Target> Resource for Buffer<Target> where Target: buffer::Target {
    type Ok = ();

    fn initialize(names: &mut [GLuint]) -> error::Result<Self::Ok> {
        unsafe {
            gl::CreateBuffers(names.len() as _, names.as_mut_ptr());
        }
        error::Error::poll_queue().map_or(Ok(()), Err)
    }

    fn free(names: &[GLuint]) -> error::Result<Self::Ok> {
        unsafe {
            gl::DeleteBuffers(names.len() as _, names.as_ptr())
        }
        error::Error::poll_queue().map_or(Ok(()), Err)
    } }
