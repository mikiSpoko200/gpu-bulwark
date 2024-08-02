#![allow(unused)]

use crate::glsl;
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
struct VertexArrayState<Attrs>
where
    Attrs: valid::Attributes,
{
    pub attributes: Attrs,
    pub length: usize,
}

impl<AS> VertexArrayState<AS>
where
    AS: valid::Attributes,
{
    pub fn vertex_attrib_pointer<A, const ATTRIBUTE_INDEX: usize>(self, vbo: Buffer<buffer::Array, A>) -> 
    VertexArrayState<(AS, Attribute<A, ATTRIBUTE_INDEX>)>
    where
        A: bounds::AttribFormat,
    {
        let attribute = Attribute::new(vbo);
        VertexArrayState {
            length: attribute.as_ref().len(),
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
    pub fn vertex_attrib_pointer<'buffer, Attr, Param, const ATTRIBUTE_INDEX: usize>(
        self,
        binding: &glsl::InBinding<Param, ATTRIBUTE_INDEX>,
        buffer: &'buffer Buffer<buffer::Array, Attr>
    ) -> VertexArray<(AS, Attribute<'buffer, Attr, ATTRIBUTE_INDEX>)>
    where
        Attr: bounds::AttribFormat,
        Param: glsl::bounds::Parameter<glsl::storage::In>
    {
        if self.phantoms.length > 0 && self.phantoms.length != buffer.state.length {
            panic!(
                "buffers must be the same length, current {} received {}",
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
}

impl VertexArray<()> {
    pub fn create() -> Self {
        Self::default()
    }
}
