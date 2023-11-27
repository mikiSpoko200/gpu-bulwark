#![allow(unused)]

use frunk::hlist::{HCons, HList, HNil};
use gl::types::GLuint;
use crate::gl_call;
use super::prelude::{Name, Object};
use super::resource::Allocator;
use super::buffer::{Buffer};
use super::buffer;
use crate::target::buffer as target;

pub struct VertexArray<Attributes> {
    object: Object<Self>,
    attributes: Attributes,
}

pub struct AttributeDecl<'buffer, Format, const INDEX: usize> {
    buffer: Buffer<target::Array, Format>
}

impl<Attributes> VertexArray<Attributes>
where
    Attributes: HList
{
    pub fn attach<'buffer, const ATTRIBUTE_INDEX: usize, Format>(self, attribute: &'buffer Buffer<target::Array, Format>)
    -> VertexArray<HCons<AttributeDecl<'buffer, Format, ATTRIBUTE_INDEX>, Attributes>>
    where
        (target::Array, Format): target::format::Valid,
    {
        let Self { object, attributes } = self;


    }
    
}

impl<Attribute> Allocator for VertexArray<Attribute> {
    type Ok = ();

    fn allocate(names: &mut [Name]) -> crate::error::Result<Self::Ok> {
        gl_call! {
            #[panic]
            // TODO: SAFETY
            unsafe {
                gl::CreateVertexArrays(
                    names.len() as _,
                    names.as_mut_ptr() as _,
                )
            }
        };
        Ok(())
    }

    fn free(names: &[Name]) -> crate::error::Result<Self::Ok> {
        todo!("unimplemented")
    }
}

pub fn draw_arrays<Attributes, Inputs>(program: (), vertex_array: VertexArray<Attributes>) {}
