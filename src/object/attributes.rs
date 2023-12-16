//! This module provides specialization of HLists for Vertex Array Object attributes.
use crate::prelude::HList;
use super::buffer::Buffer;
use crate::target;
use crate::glsl;
use crate::types;

pub(crate) struct AttributeDecl<'buffer, F, const INDEX: usize>
where
    F: Attribute,
    (target::buffer::Array, F): target::buffer::format::Valid,
{
    pub buffer: &'buffer Buffer<target::buffer::Array, F>,
}

pub trait Attribute {
    type Primitive: crate::types::Primitive;
    const SIZE: u8;
}

impl<T, const N: usize> Attribute for [T; N]
where
    T: types::Primitive,
    glsl::types::Const<N>: glsl::types::VecSize,
{
    type Primitive = T;

    const SIZE: u8 = N as _;
}

pub(crate) trait Attributes: HList {}

impl Attributes for () {}

impl<'buffer, A, AS, const INDEX: usize> Attributes for (AS, AttributeDecl<'buffer, A, INDEX>)
where
    A: Attribute,
    AS: Attributes,
    (target::buffer::Array, A): target::buffer::format::Valid,
{
}
