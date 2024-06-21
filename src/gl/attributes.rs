//! This module provides specialization of HLists for Vertex Array Object attributes.
use super::buffer::target as buffer;
use super::buffer::Buffer;
use crate::constraint;
use crate::ffi;
use crate::glsl;
use crate::glsl::marker::Scalar;
use crate::glsl::marker::Vector;
use crate::mode;
use crate::prelude::HList;
use crate::types;

pub(crate) struct AttributeDecl<'buffer, F, const INDEX: usize>
where
    F: Attribute + constraint::Valid<buffer::Array>,
{
    pub buffer: &'buffer Buffer<buffer::Array, F>,
}

#[hi::marker]
pub trait Attribute: Clone + Sized { }

impl<T, const N: usize> Attribute for [T; N]
where
    T: Clone + constraint::Valid<Scalar>,
    glsl::Const<N>: constraint::Valid<Vector>,
{
}

pub(crate) trait Attributes: HList {}

impl Attributes for () {}

impl<'buffer, A, AS, const INDEX: usize> Attributes for (AS, AttributeDecl<'buffer, A, INDEX>)
where
    A: Attribute + constraint::Valid<buffer::Array>,
    AS: Attributes,
{
}
