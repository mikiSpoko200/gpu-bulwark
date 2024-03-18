//! Shaders that contain stage entry point.

use std::marker::PhantomData;

use super::target as shader;
use super::parameters;
use crate::glsl;

use super::internal;

/// Shader that contains entry point for the stage
pub struct Main<T, IS, OS, US=()>(pub(crate) internal::CompiledShader<T>, PhantomData<IS>, PhantomData<OS>, PhantomData<US>)
where
    T: shader::Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
;

impl<T, IS, OS> Main<T, IS, OS, ()>
where
    T: shader::Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    pub(super) fn new<US>(shader: internal::CompiledShader<T>) -> Main<T, IS, OS, US> {
        Main(shader, PhantomData, PhantomData, PhantomData)
    }
}

impl<T, IS, OS, US> Main<T, IS, OS, US>
where
    T: shader::Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    pub fn input<NIS>(self) -> Main<T, (IS, NIS), OS, US>
    where
        NIS: glsl::types::Type,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn output<NOS>(self) -> Main<T, IS, (OS, NOS), US>
    where
        NOS: glsl::types::Type,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }
}
