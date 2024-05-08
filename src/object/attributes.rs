//! This module provides specialization of HLists for Vertex Array Object attributes.
use crate::prelude::HList;
use super::buffer::Buffer;
use super::buffer::target as buffer;
use crate::glsl;
use crate::types;

pub(crate) struct AttributeDecl<'buffer, F, const INDEX: usize>
where
    F: Attribute,
    (buffer::Array, F): buffer::format::Valid,
{
    pub buffer: &'buffer Buffer<buffer::Array, F>,
}

pub trait Attribute: Clone + Sized + glsl::FFI { }

impl<T, const N: usize> Attribute for [T; N]
where
    T: glsl::marker::ScalarType,
    glsl::Const<N>: glsl::VecSize,
{ }

pub(crate) trait Attributes: HList { }

impl Attributes for () { }

impl<'buffer, A, AS, const INDEX: usize> Attributes for (AS, AttributeDecl<'buffer, A, INDEX>)
where
    A: Attribute,
    AS: Attributes,
    (buffer::Array, A): buffer::format::Valid,
{ }
