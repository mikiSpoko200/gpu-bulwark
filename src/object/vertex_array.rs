#![allow(unused)]

use frunk::hlist;
use gl::types::GLuint;
use crate::gl_call;
use crate::object::prelude::{Name, Object};
use crate::object::resource::Resource;

// pub trait Attribute {
//     type Type;
//     fn size() -> usize;
//     fn relative_offset() -> usize;
// }



pub struct VertexArray<Attributes> {
    base: Object<Self>,
    attributes: Attributes,
}

impl<Attribute> Into<Object<Self>> for VertexArray<Attribute> {
    fn into(self) -> Object<Self> {
        let Self { base, .. } = self;
        base
    }
}

impl<Attribute> Resource for VertexArray<Attribute> {
    type Ok = ();

    fn initialize(names: &mut [Name]) -> crate::error::Result<Self::Ok> {
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
