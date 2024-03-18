#![allow(unused)]

use std::marker::PhantomData;

use super::buffer;
use super::buffer::Buffer;
use super::prelude::{Name, Object};
use super::resource::{Allocator, Bindable};
use crate::gl_call;
use crate::types::Primitive;
use crate::prelude::{HList, HListExt};
use super::attributes::{AttributeDecl, Attributes, Attribute};
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
struct VertexArraySemantics<AS>
where
    AS: Attributes,
{
    pub attributes: AS,
    pub length: usize,
}

impl<AS> VertexArraySemantics<AS>
where
    AS: Attributes,
{
    pub fn attach<'buffer, A, const ATTRIBUTE_INDEX: usize>(
        self,
        buffer: &'buffer Buffer<buffer::target::Array, A>,
    ) -> VertexArraySemantics<(AS, AttributeDecl<'buffer, A, ATTRIBUTE_INDEX>)>
    where
        A: Attribute,
        (buffer::target::Array, A): buffer::target::format::Valid,
    {
        let attribute = AttributeDecl { buffer };
        VertexArraySemantics {
            length: buffer.semantics.length,
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

impl<AS> VertexArray<AS>
where
    AS: Attributes,
{
    pub fn bind_buffers(&self) {
        // todo: Add Iteration over attributes to trait `Attributes`
    }

    // Idea: use curring?
    pub fn attach<'buffer, const ATTRIBUTE_INDEX: usize, A>(
        self,
        buffer: &'buffer Buffer<buffer::target::Array, A>,
    ) -> VertexArray<(AS, AttributeDecl<'buffer, A, ATTRIBUTE_INDEX>)>
    where
        A: Attribute,
        (buffer::target::Array, A): buffer::target::format::Valid,
    {
        if self.semantics.length > 0 && self.semantics.length != buffer.semantics.length {
            panic!("buffers must be the same length, current {} received {}", self.semantics.length, buffer.semantics.length);
        }

        self.bind();
        buffer.bind();
        gl_call! {
            #[panic]
            unsafe {
                gl::VertexAttribPointer(
                    ATTRIBUTE_INDEX as _,
                    A::SIZE as _,
                    <A::Primitive as Primitive>::GL_TYPE,
                    gl::FALSE,
                    0,
                    std::ptr::null()
                );
                gl::EnableVertexAttribArray(ATTRIBUTE_INDEX as _);
            }
        }
        self.unbind();
        buffer.unbind();

        let Self { object, semantics } = self;

        let mut semantics = semantics.attach(buffer);
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
