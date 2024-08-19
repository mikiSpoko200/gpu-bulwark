//! Shaders that do not contain entry point but rather contents to link against

use crate::prelude::internal::*;


use crate::gl;
use crate::ts;
use gl::shader;
use gl::uniform;

#[derive(dm::Deref)]
pub struct Lib<Target, Decls>
where
    Target: shader::Target,
    Decls: uniform::bounds::Declarations,
{
    #[deref]
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

pub type VertexLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type TCLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type TELib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type GeometryLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type FragmentLib<Decls> = Lib<shader::target::Vertex, Decls>;
pub type ComputeLib<Decls> = Lib<shader::target::Vertex, Decls>;