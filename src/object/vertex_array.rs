#![allow(unused)]

use std::marker::PhantomData;

use super::buffer;
use super::buffer::Buffer;
use super::prelude::{Name, Object};
use super::resource::{Allocator, Bindable};
use crate::gl_call;
use crate::target::buffer as target;
use crate::prelude::{HList, HListExt};
use super::attributes::{Attribute, Attributes};
use gl::types::GLuint;


struct VertexArrayAllocator;

unsafe impl Allocator for VertexArrayAllocator {
    fn allocate(names: &mut [GLuint]) {
        unsafe {
            gl::CreateVertexArrays(names.len() as _, names.as_mut_ptr());
        }
    }

    fn free(names: &[GLuint]) {
        unsafe {
            gl::DeleteVertexArrays(names.len() as _, names.as_ptr());
        }
    }
}

#[derive(Default)]
struct VertexArraySemantics<A>
where
    A: Attributes,
{
    pub length: usize,
    pub attributes: A,
}

impl<A> VertexArraySemantics<A>
where
    A: Attributes,
{
    pub fn attach<'buffer, F, const ATTRIBUTE_INDEX: usize>(
        self,
        buffer: &'buffer Buffer<target::Array, F>,
    ) -> VertexArraySemantics<(A, Attribute<'buffer, F, ATTRIBUTE_INDEX>)>
    where
        (target::Array, F): target::format::Valid,
    {
        let attribute = Attribute { buffer };
        VertexArraySemantics {
            length: self.length,
            attributes: self.attributes.append(attribute),
        }
    }
}

#[derive(Default)]
/// Representation of Vertex Array Object.
pub struct VertexArray<A>
where
    A: Attributes,
{
    object: Object<VertexArrayAllocator>,
    semantics: VertexArraySemantics<A>,
}


impl<A> VertexArray<A>
where
    A: Attributes
{
    pub const fn len(&self) -> usize {
        self.semantics.length
    }
}

impl<A> VertexArray<A>
where
    A: Attributes,
{
    // Idea: use curring?
    pub fn attach<'buffer, const ATTRIBUTE_INDEX: usize, F>(
        self,
        buffer: &'buffer Buffer<target::Array, F>,
    ) -> VertexArray<(A, Attribute<'buffer, F, ATTRIBUTE_INDEX>)>
    where
        (target::Array, F): target::format::Valid,
    {
        let Self { object, semantics } = self;
        if semantics.length != buffer.semantics.length {
            panic!("buffers must be the same length, current {} received {}", semantics.length, buffer.semantics.length);
        }
        let semantics = semantics.attach(buffer);
        VertexArray::<_> { object, semantics }
    }
}

impl VertexArray<()> {
    pub fn create() -> Self {
        Self::default()
    }
}

impl<A: Attributes> Bindable for VertexArray<A> {
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                gl::BindVertexArray(self.object.name());
            }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                gl::BindVertexArray(0);
            }
        }
    }
}
