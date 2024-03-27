//! Shaders that do not contain entry point but rather contents to link against

use std::marker::PhantomData;

use crate::object::program::uniform;

use super::internal;

use super::target as shader;
use super::CompiledShader;

pub struct Shared<T, US>(pub(crate) internal::CompiledShader<T>, PhantomData<US>)
where
    T: shader::Target,
    US: uniform::marker::LDeclarations,
;

impl<T> Shared<T, ()> 
where
    T: shader::Target,
{
    pub(super) fn new<US>(shader: internal::CompiledShader<T>) -> Shared<T, US>
    where
        US: uniform::marker::LDeclarations,
    {
        Shared(shader, PhantomData)
    }
}
