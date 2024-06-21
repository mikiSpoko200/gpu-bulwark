//! Auxiliary Type-state patten implementations.

/// Type-state for validation.
pub trait Validation { }

#[derive(Clone, Copy, Debug)]
/// Object is in unvalidated state.
pub struct Unvalidated;

#[derive(Clone, Copy, Debug)]
/// Object is in validated state.
pub struct Validated;

impl Validation for Unvalidated { }
impl Validation for Validated { }
