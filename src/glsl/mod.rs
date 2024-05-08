pub mod compatible;
mod glsl;
pub mod location;
pub mod uniform;
pub mod binding;
pub mod prelude;

pub use glsl::*;
pub use glsl::marker::{Type, VecSize};
pub use uniform::Uniform;
