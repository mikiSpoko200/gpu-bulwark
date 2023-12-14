//! This module provides specialization of HLists for Program / Shader parameters.

use crate::{prelude::HList, glsl};

/// Marker trait for types that represent program / shader inputs and outputs.
pub trait Parameters: HList { }

impl Parameters for () { }

impl<Head, T> Parameters for (Head, T)
where
    Head: Parameters,
    T: glsl::types::Type,
{ }
