//! Auxiliary Type-state patten implementations.

use crate::prelude::internal::*;

pub trait TypeState: Sized { }

/// Type-state for validation.
pub trait Validation: TypeState { }

/// Type level negation operator.
pub struct Not<TS: TypeState>(PhantomData<TS>);

#[derive(Clone, Copy, Debug)]
#[hi::mark(TypeState, Validation)]
/// Object is in unvalidated state.
pub enum Unvalidated { }

#[derive(Clone, Copy, Debug)]
#[hi::mark(TypeState, Validation)]
/// Object is in validated state.
pub enum Validated { }

#[hi::marker]
pub trait Mutability: TypeState { }

#[hi::mark(TypeState, Mutability)]
pub enum Mutable { }

#[hi::mark(TypeState, Mutability)]
pub enum Immutable { }

#[hi::marker]
pub trait Artifact { }

#[hi::marker]
pub trait Compilation: TypeState { }

#[hi::mark(TypeState, Compilation)]
pub enum Uncompiled { }

#[hi::mark(TypeState, Compilation)]
pub struct Compiled;
