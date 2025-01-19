#![allow(unused)]

use core::panic;

use crate::gl::buffer::ElementArray;
use crate::glsl;
use crate::hlist;
use crate::prelude::internal::*;

use crate::gl;
use gl::vertex_array;
use gl::buffer;
use gl::object::*;
use buffer::Buffer;
use vertex_array::valid;
use vertex_array::bounds;
use vertex_array::attribute::Attribute;

use crate::hlist::lhlist::Base as HList;

pub use glb as raw;

#[hi::mark(Object, PartialObject)]
pub enum VertexArrayObject { }

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
struct VertexArrayState<Attrs, Elem = ()>
where
    Attrs: valid::Attributes,
{
    pub attributes: Attrs,
    pub element: Buffer<ElementArray, Elem>,
    pub length: usize,
}

impl<AS> VertexArrayState<AS>
where
    AS: valid::Attributes,
{
    pub fn element_buffer<E>(self, element_buffer: Buffer<buffer::ElementArray, E>) -> VertexArrayState<AS, E>
    where
        E: buffer::_valid::ForBuffer<buffer::ElementArray>
    {
        let given = element_buffer.len();
        let expected = self.length;
        if given != expected {
            panic!("invalid element buffer length, expected: {}, got: {}", expected, given);
        }
        VertexArrayState {
            attributes: self.attributes,
            element: element_buffer,
            length: self.length,
        }
    }
}

impl<AS, E> VertexArrayState<AS, E>
where
    AS: valid::Attributes,
{
    pub fn vertex_attrib_pointer<A, const ATTRIBUTE_INDEX: usize>(self, vbo: Buffer<buffer::Array, A>) -> 
    VertexArrayState<(AS, Attribute<A, ATTRIBUTE_INDEX>), E>
    where
        A: bounds::AttribFormat,
    {
        let attribute = Attribute::new(vbo);
        VertexArrayState {
            length: attribute.as_ref().len(),
            attributes: self.attributes.append(attribute),
            element: self.element,
        }
    }
}

#[derive(Default, dm::Deref)]
/// Representation of Vertex Array Object.
pub struct VertexArray<Attrs, Elem = ()>
where
    Attrs: valid::Attributes,
{
    #[deref]
    object: ObjectBase<VertexArrayObject>,
    phantoms: VertexArrayState<Attrs, Elem>,
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
    pub fn vertex_attrib_pointer<Attr, Param, const ATTRIBUTE_INDEX: usize>(
        self,
        var: &glsl::InVariable<Param, ATTRIBUTE_INDEX>,
        buffer: Buffer<buffer::Array, Attr>
    ) -> VertexArray<(AS, Attribute<Attr, ATTRIBUTE_INDEX>)>
    where
        Attr: bounds::AttribFormat,
        Param: glsl::bounds::Parameter<glsl::storage::In>
    {
        if self.phantoms.length > 0 && self.phantoms.length != buffer.state.length {
            panic!(
                "buffers must be the same length, expected {} received {}",
                self.phantoms.length, buffer.len()
            );
        }

        let _vao_bind = self.bind();
        let _buffer_bind = buffer.bind();
        gl::call! {
            [panic]
            unsafe {
                glb::VertexAttribPointer(
                    ATTRIBUTE_INDEX as _,
                    Attr::N_COMPONENTS as _,
                    <Attr::Type as gl::Type>::ID,
                    glb::FALSE,
                    0,
                    std::ptr::null()
                );
                glb::EnableVertexAttribArray(ATTRIBUTE_INDEX as _);
            }
        }

        let Self { object, phantoms } = self;
        VertexArray { object, phantoms: phantoms.vertex_attrib_pointer(buffer) }
    }

    pub fn element_buffer<E>(self, element_buffer: Buffer<buffer::ElementArray, E>) -> VertexArray<AS, E>
    where
        E: buffer::_valid::ForBuffer<buffer::ElementArray>
    {
        let _vao_bind = self.bind();
        let _ebo_bind = element_buffer.bind();

        let given = element_buffer.len();
        let expected = self.phantoms.length;
        if given != expected {
            panic!(
                "buffers must be the same length, expected {} received {}",
                expected, given
            );
        }

        let Self { object, phantoms } = self;
        VertexArray { object, phantoms: phantoms.element_buffer(element_buffer) }
    }

    pub fn buffer_mut<Attr, Param, const ATTRIBUTE_INDEX: usize, IDX>(&mut self, var: &glsl::InVariable<Param, ATTRIBUTE_INDEX>) -> &mut Buffer<buffer::target::Array, Attr>
    where
        Attr: bounds::AttribFormat,
        Param: glsl::bounds::Parameter<glsl::storage::In>,
        IDX: hlist::counters::Index,
        AS: hlist::lhlist::Find<Attribute<Attr, ATTRIBUTE_INDEX>, IDX>,
    {
        self.phantoms.attributes.get_mut().as_mut()
    }

    pub fn buffer_ref<Attr, Param, const ATTRIBUTE_INDEX: usize, IDX>(&self, var: &glsl::InVariable<Param, ATTRIBUTE_INDEX>) -> &Buffer<buffer::target::Array, Attr>
    where
        Attr: bounds::AttribFormat,
        Param: glsl::bounds::Parameter<glsl::storage::In>,
        IDX: hlist::counters::Index,
        AS: hlist::lhlist::Find<Attribute<Attr, ATTRIBUTE_INDEX>, IDX>,
    {
        self.phantoms.attributes.get().as_ref()
    }
}

impl VertexArray<()> {
    pub fn create() -> Self {
        Self::default()
    }
}
