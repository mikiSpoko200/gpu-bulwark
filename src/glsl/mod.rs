pub mod variable;
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

pub use variable::{
    storage,
    layout,
    Storage,
    Variable,
    InVariable,
    OutVariable,
    UniformVariable,
    Qualifier,
    MatchingInputs,
    vars,
    uniforms,
    Uniforms,
    inputs,
    Inputs,
    outputs,
    Outputs,
};
