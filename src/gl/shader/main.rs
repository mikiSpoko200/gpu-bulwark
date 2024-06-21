//! Shaders that contain stage entry point.

#![recursion_limit = "10"]

use std::marker::PhantomData;

use super::target as shader;
use crate::gl::program::uniform;
use crate::glsl;
use crate::glsl::prelude::*;
use crate::hlist;
use crate::hlist::indexed::lhlist;
use crate::hlist::indexed::lhlist::Append;

use super::internal;
use glsl::storage;


/// Shader that contains entry point for the stage
pub struct Main<T, IS, OS, US = ()>(pub(crate) internal::CompiledShader<T>, PhantomData<(IS, OS, US)>)
where
    T: shader::Target,
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    US: uniform::marker::Declarations,
;

impl<T, IS, OS> Main<T, IS, OS, ()>
where
    T: shader::Target,
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
{
    pub(super) fn new<US>(shader: internal::CompiledShader<T>) -> Main<T, IS, OS, US>
    where
        US: uniform::marker::Declarations,
    {
        Main(shader, PhantomData)
    }
}

impl<T, IS, OS, US> Main<T, IS, OS, US>
where
    T: shader::Target,
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    US: uniform::marker::Declarations,
{
    pub fn input<NIS, const LOCATION: usize>(self, _: &InParameterBinding<NIS, LOCATION>) -> Main<T, (IS, InParameterBinding<NIS, LOCATION>), OS, US>
    where
        NIS: glsl::Type,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn output<NOS, const LOCATION: usize>(self, _: &OutParameterBinding<NOS, LOCATION>) -> Main<T, IS, (OS, OutParameterBinding<NOS, LOCATION>), US>
    where
        NOS: glsl::Type,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn inputs<NIS>(self, inputs: &NIS) -> Main<T, IS::Concatenated, OS, US>
    where
        IS: hlist::lhlist::Concatenate<NIS>,
        IS::Concatenated: glsl::Parameters<storage::In>,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn outputs<NOS>(self, inputs: &NOS) -> Main<T, IS, OS::Concatenated, US>
    where
        OS: hlist::lhlist::Concatenate<NOS>,
        OS::Concatenated: glsl::Parameters<storage::Out>,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }
}
