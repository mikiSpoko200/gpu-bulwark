pub mod binding;
pub mod compatible;
mod glsl;
pub mod location;
pub mod parameters;
pub mod prelude;
pub mod sampler;
pub mod uniform;
pub mod qualifier;

pub use glsl::marker::Type;
pub use glsl::*;
pub use parameters::Parameters;
pub use uniform::{marker::Uniforms, Uniform};
pub use binding::marker::{storage, layout};
pub use compatible::Compatible;
