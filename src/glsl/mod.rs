pub mod binding;
pub mod compatible;
mod glsl;
pub mod location;
pub mod parameters;
pub mod sampler;
pub mod uniform;
pub mod qualifier;
pub mod bounds;
pub mod valid;

pub use location::Location;
pub use glsl::Type;
pub use glsl::*;
pub use parameters::Parameters;
pub use uniform::{Uniforms, Uniform};
pub use compatible::Compatible;

pub use binding::{
    storage,
    layout,
    Storage,
    Binding,
    InBinding,
    OutBinding,
    UniformBinding,
    Qualifier,
    MatchingInputs,
};