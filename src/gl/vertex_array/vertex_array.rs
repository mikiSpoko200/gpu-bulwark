#![allow(unused)]

use crate::glsl;
use crate::prelude::internal::*;

use crate::gl;
use crate::valid;
use gl::buffer;

use gl::object::*;
use buffer::Buffer;
use super::attribute::{Attribute};

use crate::hlist::lhlist::Base as HList;
use crate::types::Primitive;

#[hi::mark(Object, PartialObject)]
enum VertexArrayObject { }

unsafe impl Allocator for VertexArrayObject {
    fn allocate(names: &mut [u32]) {
        gl::call! {
            [panic]
            unsafe {
                glb::CreateVertexArrays(names.len() as _, names.as_mut_ptr());
            }
        }
    }

    fn free(names: &[u32]) {
        gl::call! {
            [panic]
            unsafe {
                glb::DeleteVertexArrays(names.len() as _, names.as_ptr());
            }
        }
    }
}

impl Binder for VertexArrayObject {
    fn bind(name: u32) {
        gl::call! {
            [panic]
            unsafe {
                glb::BindVertexArray(name);
            }
        }
    }
}

#[derive(Default)]
struct VertexArrayState<AS>
where
    AS: valid::Attributes,
{
    pub attributes: AS,
    pub length: usize,
}

impl<AS> VertexArrayState<AS>
where
    AS: valid::Attributes,
{
    pub fn attach<'buffer, A, const ATTRIBUTE_INDEX: usize>(self,buffer: &'buffer Buffer<buffer::Array, A>,) -> VertexArrayState<(AS, Attribute<'buffer, A, ATTRIBUTE_INDEX>)>
    where
        A: valid::ForAttribute,
    {
        let attribute = Attribute { buffer };
        VertexArrayState {
            length: buffer.state.length,
            attributes: self.attributes.append(attribute),
        }
    }
}

#[derive(Default, dm::Deref)]
/// Representation of Vertex Array Object.
pub struct VertexArray<Attrs>
where
    Attrs: valid::Attributes,
{
    #[deref]
    object: ObjectBase<VertexArrayObject>,
    phantoms: VertexArrayState<Attrs>,
}

pub type VAO<Attrs> = VertexArray<Attrs>;

impl<Attrs: valid::Attributes> VertexArray<Attrs> {
    pub const fn len(&self) -> usize {
        self.phantoms.length
    }
}

impl<AS> VertexArray<AS>
where
    AS: valid::Attributes,
{
    pub fn bind_buffers(&self) {
        // todo: Add Iteration over attributes to trait `Attributes`
    }

    // TODO: Fix type var validation. On both glsl and gl
    pub fn attach<'buffer, A, const ATTRIBUTE_INDEX: usize>(
        self,
        binding: &glsl::binding::InBinding<A, ATTRIBUTE_INDEX>,
        buffer: &'buffer Buffer<buffer::Array, impl >
    ) -> VertexArray<(AS, Attribute<'buffer, A, ATTRIBUTE_INDEX>)>
    where
        A: valid::ForAttribute,
    {
        if self.phantoms.length > 0 && self.phantoms.length != buffer.state.length {
            panic!(
                "buffers must be the same length, current {} received {}",
                self.phantoms.length, buffer.state.length
            );
        }

        let vao_bind = self.bind();
        let buffer_bind = buffer.bind();
        buffer.bind();
        gl::call! {
            [panic]
            unsafe {
                glb::VertexAttribPointer(
                    ATTRIBUTE_INDEX as _,
                    A::N_COMPONENTS as _,
                    <A::Primitive as Primitive>::GL_TYPE,
                    glb::FALSE,
                    0,
                    std::ptr::null()
                );
                glb::EnableVertexAttribArray(ATTRIBUTE_INDEX as _);
            }
        }

        let Self { object, phantoms } = self;

        VertexArray { object, phantoms: phantoms.attach(buffer) }
    }
}

impl VertexArray<()> {
    pub fn create() -> Self {
        Self::default()
    }
}
