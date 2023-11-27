use crate::{impl_const_super_trait, prelude::TypeEnum};

/// Definitions of opengl bind targets
pub mod buffer;
pub mod prelude;

/// Common behaviour amongst all object specific targets
pub unsafe trait Target: TypeEnum { }
