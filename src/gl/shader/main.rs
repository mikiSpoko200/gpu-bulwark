//! Shaders that contain stage entry point.

use crate::prelude::internal::*;

use crate::gl;
use crate::glsl;
use crate::ts;
use crate::valid;
use crate::hlist;
use crate::hlist::indexed::lhlist;
use crate::hlist::indexed::lhlist::Append;
use gl::uniform;
use gl::shader::target;

use glsl::storage;
use glsl::prelude::*;

/// Shader that contains entry point for the stage
pub struct Main<Target, Ins, Outs, Decls>(pub(crate) super::CompiledShader<Target, Decls>, PhantomData<(Ins, Outs)>)
where
    Target: target::Target,
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
;

impl<Target, Ins, Outs, Decls> std::ops::Deref for Main<Target, Ins, Outs, Decls>
where
    Target: target::Target,
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    type Target = super::Shader<ts::Compiled, Target, Decls>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Target, Ins, Outs> Main<Target, Ins, Outs, ()>
where
    Target: target::Target,
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
{
    pub(super) fn new<Decls>(shader: super::CompiledShader<Target, Decls>) -> Main<Target, Ins, Outs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        Main(shader, PhantomData)
    }
}

impl<Target, Ins, Outs, Decls> Main<Target, Ins, Outs, Decls>
where
    Target: target::Target,
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    pub fn input<In, const LOCATION: usize>(self, _: &InBinding<In, LOCATION>) -> Main<Target, (Ins, InBinding<In, LOCATION>), Outs, Decls>
    where
        In: valid::ForAttribute,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn output<Out, const LOCATION: usize>(self, _: &OutBinding<Out, LOCATION>) -> Main<Target, Ins, (Outs, OutBinding<Out, LOCATION>), Decls>
    where
        Out: valid::ForAttribute,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn inputs<NIns>(self, inputs: &NIns) -> Main<Target, Ins::Concatenated, Outs, Decls>
    where
        Ins: hlist::lhlist::Concatenate<NIns>,
        Ins::Concatenated: glsl::Parameters<storage::In>,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }

    pub fn outputs<NOuts>(self, inputs: &NOuts) -> Main<Target, Ins, Outs::Concatenated, Decls>
    where
        Outs: hlist::lhlist::Concatenate<NOuts>,
        Outs::Concatenated: glsl::Parameters<storage::Out>,
    {
        let Self(shader, ..) = self;
        Main::new(shader)
    }
}

pub type VertexMain<Ins, Outs, Decls> = Main<target::Vertex, Ins, Outs, Decls>;
pub type TEMain<Ins, Outs, Decls> = Main<target::tesselation::Control, Ins, Outs, Decls>;
pub type TCMain<Ins, Outs, Decls> = Main<target::tesselation::Evaluation, Ins, Outs, Decls>;
pub type GeometryMain<Ins, Outs, Decls> = Main<target::Geometry, Ins, Outs, Decls>;
pub type FragmentMain<Ins, Outs, Decls> = Main<target::Fragment, Ins, Outs, Decls>;
pub type ComputeMain<Ins, Outs, Decls> = Main<target::Compute, Ins, Outs, Decls>;
