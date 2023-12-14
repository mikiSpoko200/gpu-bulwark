//! This module provides specialization of HLists for Vertex Array Object attributes.
use crate::prelude::HList;
use super::buffer::Buffer;
use crate::target;

pub(crate) struct Attribute<'buffer, F, const INDEX: usize>
where
    (target::buffer::Array, F): target::buffer::format::Valid,
{
    pub buffer: &'buffer Buffer<target::buffer::Array, F>,
}

pub(crate) trait Attributes: HList {}

impl Attributes for () {}

impl<'buffer, F, const INDEX: usize, Tail> Attributes for (Tail, Attribute<'buffer, F, INDEX>)
where
    Tail: Attributes,
    (target::buffer::Array, F): target::buffer::format::Valid,
{
}
