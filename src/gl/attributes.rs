//! This module provides specialization of HLists for Vertex Array Object attributes.
use super::buffer;
use buffer::target;
use buffer::Buffer;
use crate::constraint;
use crate::ffi;
use crate::glsl;
use crate::valid;
use crate::mode;
use crate::types;
use crate::hlist;

use crate::prelude::internal::*;

pub(crate) struct AttributeDecl<'buffer, F, const INDEX: usize>
where
    F: Attribute + valid::ForBuffer<target::Array>,
{
    pub buffer: &'buffer Buffer<target::Array, F>,
}

#[hi::marker]
pub trait Attribute: valid::ForBuffer<target::Array> { }

impl<T, const N: usize> Attribute for [T; N]
where
    T: glsl::bounds::ScalarType,
    Const<N>: valid::VecDim,
{
}

pub(crate) trait Attributes: crate::hlist::HList { }

impl Attributes for () { }

impl<'buffer, A, AS, const INDEX: usize> Attributes for (AS, AttributeDecl<'buffer, A, INDEX>)
where
    A: Attribute + valid::ForArray,
    AS: Attributes,
{
}
