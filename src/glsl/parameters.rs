//! This module provides specialization of HLists for Program / Shader parameters.

use crate::glsl;
use crate::hlist::lhlist::Base as HList;
use glsl::binding::{InBinding, OutBinding, Storage, marker::storage};

use super::prelude::{marker::storage::{In, Out}, Qualifier};

/// Marker trait for types that represent program / shader inputs and outputs.
pub trait Parameters<Q>: HList
where
    Q: Qualifier<Storage>,
{
}

impl<Q> Parameters<Q> for () where Q: Qualifier<Storage> {}

impl<Head, T, const LOCATION: usize> Parameters<storage::In> for (Head, InBinding<T, LOCATION>)
where
    Head: Parameters<In>,
    T: glsl::Type,
{
}

impl<Head, T, const LOCATION: usize> Parameters<storage::Out> for (Head, OutBinding<T, LOCATION>)
where
    Head: Parameters<Out>,
    T: glsl::Type,
{
}
