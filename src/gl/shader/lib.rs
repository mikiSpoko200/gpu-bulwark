//! Shaders that do not contain entry point but rather contents to link against

use crate::prelude::internal::*;


use crate::gl;
use crate::ts;
use gl::shader;
use gl::uniform;

use super::internal;


pub struct Lib<Target, Decls>
where
    Target: shader::Target,
    Decls: uniform::bounds::Declarations,
{
    inner: super::CompiledShader<Target, Decls>,
}

impl<Target, Decls> Lib<Target, Decls>
where
    Target: shader::Target,
    Decls: uniform::bounds::Declarations,
{
    pub(super) fn new(shader: super::CompiledShader<Target, Decls>) -> Self {
        Lib {
            inner: shader, 
        }
    }
}

impl<Target, Decls> std::ops::Deref for Lib<Target, Decls>
where 
    Target: gl::shader::target::Target, 
    Decls: gl::uniform::bounds::Declarations
{
    type Target = super::Shader<ts::Compiled, Target, Decls>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub type VertexLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type TCLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type TELib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type GeometryLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type FragmentLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type ComputeLib<Decls> = Lib<shader::target::Vertex, Decls>;