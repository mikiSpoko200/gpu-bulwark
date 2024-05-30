//! This module provides specialization of HLists for Program / Shader parameters.

use crate::glsl;
use crate::hlist::lhlist::Base as HList;
use glsl::binding::{In, InParameterBinding, Out, OutParameterBinding, ParameterStorageQualifier};

/// Marker trait for types that represent program / shader inputs and outputs.
pub trait Parameters<Qualifier>: HList
where
    Qualifier: ParameterStorageQualifier,
{
}

impl<Q> Parameters<Q> for () where Q: ParameterStorageQualifier {}

impl<Head, T, const LOCATION: usize> Parameters<In> for (Head, InParameterBinding<T, LOCATION>)
where
    Head: Parameters<In>,
    T: glsl::Type,
{
}

impl<Head, T, const LOCATION: usize> Parameters<Out> for (Head, OutParameterBinding<T, LOCATION>)
where
    Head: Parameters<Out>,
    T: glsl::Type,
{
}
