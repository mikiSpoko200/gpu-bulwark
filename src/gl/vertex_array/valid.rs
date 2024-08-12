use crate::prelude::internal::*;

use crate::gl::vertex_array;
use vertex_array::attribute::Attribute;
use vertex_array::bounds;

#[hi::marker]
pub trait Attributes: crate::hlist::lhlist::Base { }

impl Attributes for () { }

impl<H, Attr, const ATTRIB_INDEX: usize> Attributes for (H, Attribute<Attr, ATTRIB_INDEX>)
where
    H: Attributes,
    Attr: bounds::AttribFormat
{ }
