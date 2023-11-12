use std::marker::PhantomData;
use crate::object::shader::{Shader, Stage};

pub mod layout;

pub trait ShaderInterface { }

// pub const fn locations(shader_interface: impl ShaderInterface) { }

struct Entry<S>(pub Shader<S>)
    where
        S: Stage;

struct Subroutines<S>(pub Vec<Shader<S>>)
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
