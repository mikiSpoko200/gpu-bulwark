use crate::prelude::internal::*;

pub mod attribute;
pub mod vertex_array;
pub mod valid;
pub mod bounds;
pub mod format;
pub mod binding;
pub mod layout;

// public Re-exports

pub use vertex_array::*;
pub use format::Format;
pub use binding::VertexBufferBinding;
pub use attribute::Attribute;

#[macro_export]
macro_rules! Attributes {
    () => {
        
    };
}

pub use Attributes;