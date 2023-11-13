use crate::object::shader::{Compiled, Shader, Stage};
use std::marker::PhantomData;

pub mod layout;

pub trait ShaderInterface {}

// pub const fn locations(shader_interface: impl ShaderInterface) { }

struct Entry<S>(pub Shader<S, Compiled>)
where
    S: Stage;

struct Subroutines<S>(pub Vec<Shader<S, Compiled>>)
where
    S: Stage;

/// Collection of shaders for given program stage with defined stage interface.
///
/// It contains exactly one shaders that contains main function
/// and arbitrary many that are there just to supply subroutines.
pub struct ShaderStage<S, Inputs, Outputs>
where
    S: Stage,
{
    entry: Entry<S>,
    subroutines: Subroutines<S>,
    _in_phantom: PhantomData<Inputs>,
    _out_phantom: PhantomData<Outputs>,
}
