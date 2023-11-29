#![allow(unused)]

use std::marker::PhantomData;

use super::buffer;
use super::buffer::Buffer;
use super::prelude::{Name, Object};
use super::resource::Allocator;
use crate::gl_call;
use crate::target::buffer as target;
use frunk::hlist::{HCons, HList, HNil};
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
    A: HList,
{
    pub attributes: A,
}

impl<A> VertexArraySemantics<A>
where
    A: HList,
{
    pub fn attach<T>(self, attribute: T) -> VertexArraySemantics<HCons<T, A>> {
        VertexArraySemantics {
            attributes: self.attributes.prepend(attribute),
        }
    }
}

#[derive(Default)]
pub struct VertexArray<A>
where
    A: HList,
{
    object: Object<VertexArrayAllocator>,
    semantics: VertexArraySemantics<A>,
}

pub struct AttributeDecl<'buffer, F, const INDEX: usize>
where
    (target::Array, F): target::format::Valid,
{
    buffer: &'buffer Buffer<target::Array, F>,
}

impl<A> VertexArray<A>
where
    A: HList,
{
    // Idea: use curring?
    pub fn attach<'buffer, const ATTRIBUTE_INDEX: usize, F>(
        self,
        buffer: &'buffer Buffer<target::Array, F>,
    ) -> VertexArray<HCons<AttributeDecl<'buffer, F, ATTRIBUTE_INDEX>, A>>
    where
        (target::Array, F): target::format::Valid,
    {
        let Self { object, semantics } = self;
        let attribute: AttributeDecl<'buffer, F, ATTRIBUTE_INDEX> = AttributeDecl { buffer };
        let semantics = semantics.attach(attribute);
        VertexArray::<_> { object, semantics }
    }
}

impl VertexArray<HNil> {
    pub fn create() -> Self {
        Self::default()
    }
}

pub fn draw_arrays<A, I>(program: (), vertex_array: VertexArray<A>)
where
    A: HList,
{
}
