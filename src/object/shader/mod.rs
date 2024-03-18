pub(super) mod internal;
pub mod target;
pub mod main;
pub mod shared;
pub mod parameters;
pub mod prelude;

use target as shader;

use super::prelude::*;
use super::program::uniform;
use crate::object::resource::Allocator;
use crate::prelude::*;
use crate::{gl_call, impl_const_super_trait};
use crate::glsl;
use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

pub(super) use main::Main;
pub(super) use shared::Shared;

use crate::object::resource;
use thiserror;

#[derive(thiserror::Error, Debug)]
#[error("shader compilation failed {msg}")]
pub struct CompilationError {
    pub msg: String,
}

impl CompilationError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

pub trait CompilationStatus {}

#[derive(Default)]
pub struct Uncompiled;
impl CompilationStatus for Uncompiled {}

pub struct Compiled;
impl CompilationStatus for Compiled {}

pub struct Declarations<T>(pub(crate) PhantomData<T>);

impl<T> Declarations<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for Declarations<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Definitions<T>(pub(crate) T);

impl<T> Definitions<T> {
    pub fn new(t: T) -> Self {
        Self(t)
    }
}

pub struct Shader<T, C = Uncompiled, US = ()>
where
    T: shader::Target,
    C: CompilationStatus,
    US: uniform::marker::Declarations
{
    internal: internal::Shader<T, C>,
    _uniform_declarations: Declarations<US>,
}

// TODO: move this to internal and recreate here ones with US parameter
pub type CompiledShader<T, US> = Shader<T, Compiled, US>;

pub type VertexShader<US> = CompiledShader<target::Vertex, US>;
pub type TesselationControlShader<US> = CompiledShader<target::tesselation::Control, US>;
pub type TesselationEvaluationShader<US> = CompiledShader<target::tesselation::Evaluation, US>;
pub type GeometryShader<US> = CompiledShader<target::Geometry, US>;
pub type FragmentShader<US> = CompiledShader<target::Fragment, US>;
pub type ComputeShader<US> = CompiledShader<target::Compute, US>;

impl<T> Shader<T, Uncompiled, ()>
where
    T: shader::Target
{
    pub fn create() -> Self {
        Self {
            _uniform_declarations: Declarations::default(),
            internal: internal::Shader::create(),
        }
    }
}

impl<T> Default for Shader<T, Uncompiled, ()>
where
    T: shader::Target,
{
    fn default() -> Self {
        Self::create()
    }
}

impl<T, US> Shader<T, Uncompiled, US>
where
    T: shader::Target,
    US: uniform::marker::Declarations
{
    /// Add source for shader.
    pub fn source(&self, sources: &[&str]) -> &Self {
        self.internal.source(sources);
        self
    }

    pub fn compile(self) -> Result<Shader<T, Compiled, US>, CompilationError> {
        let compiled_internal = self.internal.compile()?;
        Ok(Shader {
            internal: compiled_internal,
            _uniform_declarations: self._uniform_declarations,
        })
    }
}

impl<T, US> Shader<T, Compiled, US>
where
    T: shader::Target,
    US: uniform::marker::Declarations
{
    pub fn into_main(self) -> Main<T, (), (), US> {
        Main::new(self.internal)
    }

    pub fn into_shared(self) -> Shared<T, US> {
        Shared::new(self.internal)
    }
}

pub trait TargetProvider {
    type Target: shader::Target;
}
