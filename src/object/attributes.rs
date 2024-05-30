//! This module provides specialization of HLists for Vertex Array Object attributes.
use super::buffer::target as buffer;
use super::buffer::Buffer;
use crate::glsl;
use crate::prelude::HList;
use crate::types;

pub(crate) struct AttributeDecl<'buffer, F, const INDEX: usize>
where
    F: Attribute + buffer::format::Valid<buffer::Array>,
{
    pub buffer: &'buffer Buffer<buffer::Array, F>,
}

pub trait Attribute: Clone + Sized + glsl::FFI {}

impl<T, const N: usize> Attribute for [T; N]
where
    T: glsl::marker::ScalarType,
    glsl::Const<N>: glsl::VecSize,
{
}

pub(crate) trait Attributes: HList {}

impl Attributes for () {}

impl<'buffer, A, AS, const INDEX: usize> Attributes for (AS, AttributeDecl<'buffer, A, INDEX>)
where
    A: Attribute + buffer::format::Valid<buffer::Array>,
    AS: Attributes,
{
}
