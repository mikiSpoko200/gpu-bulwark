#![allow(unused)]

use std::marker::PhantomData;

use super::attributes::{Attribute, AttributeDecl, Attributes};
use super::buffer;
use super::buffer::Buffer;
use super::prelude::{Name, Object};
use super::resource::{Allocator, Bind};
use crate::{constraint, gl_call, mode};
use crate::hlist::HList;
use crate::types::Primitive;
use glb::types::GLuint;

struct VertexArrayAllocator;

unsafe impl Allocator for VertexArrayAllocator {
    fn allocate(names: &mut [GLuint]) {
        unsafe {
            glb::CreateVertexArrays(names.len() as _, names.as_mut_ptr());
        }
    }

    fn free(names: &[GLuint]) {
        unsafe {
            glb::DeleteVertexArrays(names.len() as _, names.as_ptr());
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
    A: Attributes,
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

    pub fn attach<'buffer, const ATTRIBUTE_INDEX: usize, A>(
        self,
        buffer: &'buffer Buffer<buffer::target::Array, A>,
    ) -> VertexArray<(AS, AttributeDecl<'buffer, A, ATTRIBUTE_INDEX>)>
    where
        A: Attribute,
    {
        if self.semantics.length > 0 && self.semantics.length != buffer.semantics.length {
            panic!(
                "buffers must be the same length, current {} received {}",
                self.semantics.length, buffer.semantics.length
            );
        }

        self.bind();
        buffer.bind();
        gl_call! {
            #[panic]
            unsafe {
                glb::VertexAttribPointer(
                    ATTRIBUTE_INDEX as _,
                    A::SIZE as _,
                    <A::Primitive as Primitive>::GL_TYPE,
                    glb::FALSE,
                    0,
                    std::ptr::null()
                );
                glb::EnableVertexAttribArray(ATTRIBUTE_INDEX as _);
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

impl<A: Attributes> Bind for VertexArray<A> {
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                glb::BindVertexArray(self.object.name());
            }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                glb::BindVertexArray(0);
            }
        }
    }
}
