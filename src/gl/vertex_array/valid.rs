use crate::prelude::internal::*;


pub trait ForAttribute {
    const N_COMPONENTS: usize;
}

#[hi::marker]
pub(crate) trait Attributes: crate::hlist::lhlist::Base { }

impl Attributes for () { }