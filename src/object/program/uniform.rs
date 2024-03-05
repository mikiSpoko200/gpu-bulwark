//! This module provides specialization of HLists for Program Uniforms.

use crate::{prelude::HList, glsl};

pub unsafe trait Uniform: Location {
    
}

/// Type collections that represent program uniforms.
pub trait Uniforms: HList { }


impl Uniforms for () { }

impl<Head, T> Uniforms for (Head, T)
where
    Head: Uniforms,
    T: glsl::types::Type,
{ }


pub struct Uniforms<US>(US);

