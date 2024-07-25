use std::marker::PhantomData;

use marker::storage::{In, Out};

use super::internal;
use super::shader;
use super::shader::prelude::*;
use super::uniform;
use super::uniform::Definitions;
use super::uniform::Matcher;
use crate::glsl;
use crate::glsl::binding;
use crate::glsl::prelude::*;
use crate::hlist;
use crate::hlist::lhlist;
use crate::ts;
use crate::utils;

pub struct Data<Ins, Outs>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
{
    inputs: PhantomData<Ins>,
    outputs: PhantomData<Outs>,
}

impl<Ins, Outs> Default for Data<Ins, Outs>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
{
    fn default() -> Self {
        Self {
            inputs: Default::default(),
            outputs: Default::default(),
        }
    }
}

pub struct Builder<'shaders, Target, Ins, Outs, Defs, Decls>
where
    Target: shader::target::Target,
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
    Decls: uniform::bounds::Declarations,
{
    _target_phantom: PhantomData<Target>,
    _data: Data<Ins, Outs>,
    matcher: Option<uniform::Matcher<Defs, Decls>>,
    vertex: Option<internal::ShaderStage<'shaders, Vertex>>,
    tesselation_control: Option<internal::ShaderStage<'shaders, tesselation::Control>>,
    tesselation_evaluation: Option<internal::ShaderStage<'shaders, tesselation::Evaluation>>,
    geometry: Option<internal::ShaderStage<'shaders, Geometry>>,
    fragment: Option<internal::ShaderStage<'shaders, Fragment>>,
    compute: Option<internal::ShaderStage<'shaders, Compute>>,
}

impl<'s, Target, Ins, Outs, Defs> Builder<'s, Target, Ins, Outs, Defs, ()>
where
    Target: shader::target::Target,
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
{
    /// Update type parameters on `Main` shader attachment.
    /// 
    /// `Main` shader attachment advances Builder's `Target`, `Outs` and `Decls` parameters.
    fn attach_main<NTarget, NOuts, Decls>(self, decls: uniform::Declarations<ts::Mutable, Decls>) -> Builder<'s, NTarget, Ins, NOuts, Defs, Decls>
    where
        NTarget: shader::target::Target,
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            matcher: self.matcher.map(|inner|inner.set_declarations(decls)),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    /// Update type parameters on `Lib` shader attachment.
    /// 
    /// `Shared` shader can require some additional uniforms.
    fn attach_lib<Decls>(self, decls: uniform::Declarations<ts::Mutable, Decls>) -> Builder<'s, Vertex, Ins, Outs, Defs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: self._data,
            matcher: self.matcher.map(|inner|inner.set_declarations(decls)),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    /// Vertex shader attachment is different as it also sets `Ins` (from initially empty list).
    fn attach_vertex_main<NIns, NOuts, Decls>(self, decls: uniform::Declarations<ts::Mutable, Decls>) -> Builder<'s, Vertex, NIns, NOuts, Defs, Decls>
    where
        NIns: glsl::Parameters<In>,
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            matcher: self.matcher.map(|inner|inner.set_declarations(decls)),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }
}

impl<'s, T, Ins, Outs, Defs, Decls> Builder<'s, T, Ins, Outs, Defs, Decls>
where
    T: shader::target::Target,
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
    Decls: uniform::bounds::Declarations,
{
    /// Map uniform declarations from most recently attached shader to definitions provided by the program. 
    pub fn uniforms(self, matcher: impl FnOnce(Matcher<Defs, Decls>) -> Matcher<Defs, ()>) -> Builder<'s, T, Ins, Outs, Defs, ()> {
        Builder {
            _target_phantom: PhantomData,
            _data: self._data,
            matcher: self.matcher.map(matcher),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }
}

impl<'s, Defs> Builder<'s, Vertex, (), (), Defs, ()>
where
    Defs: uniform::bounds::Definitions,
{
    /// Create new Builder and provide uniform definitions.
    pub fn new(definitions: uniform::Definitions<Defs>) -> Self {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            matcher: Some(uniform::Matcher::new(definitions)),
            vertex: None,
            tesselation_control: None,
            tesselation_evaluation: None,
            geometry: None,
            fragment: None,
            compute: None,
        }
    }
}

/// impl for initial stage
impl<'s, Defs> Builder<'s, Vertex, (), (), Defs, ()>
where
    Defs: uniform::bounds::Definitions,
{
    pub fn vertex_main<VIns, VOuts, Decls>(mut self, vertex: &'s super::Main<Vertex, VIns, VOuts, Decls>) -> Builder<Vertex, VIns, VOuts, Defs, Decls>
    where
        VIns: super::glsl::Parameters<In>,
        VOuts: super::glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.vertex = Some(internal::ShaderStage::new(&vertex));
        self.attach_vertex_main(vertex.declarations())
    }
}

/// impl for vertex stage
impl<'s, Ins, Outs, Defs> Builder<'s, Vertex, Ins, Outs, Defs, ()>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    /// Attach new vertex shader for linking purposes possibly adding new uniforms.
    pub fn vertex_shared<Decls>(mut self, vertex: &'s Lib<Vertex, Decls>) -> Builder<'_, Vertex, Ins, Outs, Defs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.vertex
            .as_mut()
            .expect("vertex stage is set")
            .libs
            .push(vertex.as_ref());
        self.attach_lib(vertex.declarations())
    }

    pub fn tesselation_control_main<NOuts, Decls>(mut self, tesselation_control: &'s Main<tesselation::Control, Outs::Inputs, NOuts, Decls>) -> Builder<tesselation::Control, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.tesselation_control = Some(internal::ShaderStage::new(&tesselation_control.0));
        self.attach_main(tesselation_control.declarations())
    }

    pub fn geometry_main<NOuts, Decls>(mut self, geometry: &'s Main<Geometry, Outs::Inputs, NOuts, Decls>) -> Builder<Geometry, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.geometry = Some(internal::ShaderStage::new(&geometry.0));
        self.attach_main(geometry.declarations())
    }

    pub fn fragment_main<NOuts, Decls>(mut self, fragment: &'s Main<Fragment, Outs::Inputs, NOuts, Decls>) -> Builder<Fragment, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.fragment
            .replace(internal::ShaderStage::new(&fragment.0));
        self.attach_main(fragment.declarations())
    }
}

/// impl for tesselation control stage
impl<'s, Ins, Outs, Defs> Builder<'s, tesselation::Control, Ins, Outs, Defs, ()>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    pub fn tesselation_control_shared<Decls>(mut self, tesselation_control: &'s Lib<tesselation::Control, Decls>) -> Builder<'_, tesselation::Control, Ins, Outs, Defs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.tesselation_control
            .as_mut()
            .expect("tesselation control was initialized")
            .libs
            .push(tesselation_control.as_ref());
        self.attach_main(tesselation_control.declarations())
    }

    pub fn tesselation_evaluation_main<NOuts, Decls>(mut self, te_main: &'s Main<tesselation::Evaluation, Outs::Inputs, NOuts, Decls>) -> Builder<tesselation::Evaluation, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.tesselation_evaluation = Some(internal::ShaderStage::new(te_main));
        self.attach_main(te_main.declarations())
    }
}

/// impl for tesselation evaluation stage
impl<'s, Ins, Outs, Defs> Builder<'s, tesselation::Evaluation, Ins, Outs, Defs, ()>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    pub fn tesselation_evaluation_shared<Decls>(mut self, te_lib: &'s Lib<tesselation::Evaluation, Decls>) -> Builder<'_, tesselation::Evaluation, Ins, Outs, Defs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.tesselation_evaluation
            .as_mut()
            .expect("tesselation evaluation stage was initialized")
            .libs
            .push(te_lib.as_ref());
        self.attach_main(te_lib.declarations())
    }

    pub fn geometry_main<NOuts, Decls>(mut self, geometry: &'s Main<Geometry, Outs::Inputs, NOuts, Decls>) -> Builder<Geometry, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.geometry = Some(internal::ShaderStage::new(geometry));
        self.attach_main(geometry.declarations())
    }

    pub fn fragment_main<NOuts, Decls>(mut self, fragment: &'s Main<Fragment, Outs::Inputs, NOuts, Decls>) -> Builder<Fragment, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.fragment = Some(internal::ShaderStage::new(&fragment.0));
        self.attach_main(fragment.declarations())
    }
}

/// impl for geometry stage
impl<'s, Ins, Outs, Defs> Builder<'s, Geometry, Ins, Outs, Defs, ()>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    pub fn geometry_shared<Decls>(mut self, geometry: &'s Lib<Geometry, Decls>) -> Builder<Geometry, Ins, Outs, Defs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.geometry
            .as_mut()
            .expect("geometry stage was initialized")
            .libs
            .push(geometry.as_ref());
        self.attach_main(geometry.declarations())
    }

    pub fn fragment_main<NOuts, Decls>(mut self, fragment: &'s Main<Fragment, Outs::Inputs, NOuts, Decls>) -> Builder<Fragment, Ins, NOuts, Defs, Decls>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.fragment = Some(internal::ShaderStage::new(&fragment.0));
        self.attach_main(fragment.declarations())
    }
}

/// impl for fragment stage
impl<'s, Ins, Outs, Defs> Builder<'s, Fragment, Ins, Outs, Defs, ()>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
{
    pub fn fragment_shared<Decls>(mut self, fragment: &'s Lib<Fragment, Decls>) -> Builder<'_, Fragment, Ins, Outs, Defs, Decls>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.fragment
            .as_mut()
            .expect("fragment stage was initialized")
            .libs
            .push(fragment.as_ref());
        self.attach_main(fragment.declarations())
    }

    /// Build `Program` by linking all the provided attachments.
    pub fn build(&self) -> Result<super::Program<Ins, Outs, Defs::AsDeclarations>, super::LinkingError> {
        let program = super::Program::create_with_uniforms(&self.matcher.expect("matcher is provided").definitions);

        program.attach(self.vertex.as_ref().expect("vertex shader stage is set"));

        if let (Some(control_stage), Some(evaluation_stage)) =
            (&self.tesselation_control, &self.tesselation_evaluation)
        {
            program.attach(control_stage);
            program.attach(evaluation_stage);
        }

        if let Some(geometry) = &self.geometry {
            program.attach(geometry);
        }

        program.attach(
            self.fragment
                .as_ref()
                .expect("fragment shader stage is set"),
        );
        program.link()
    }
}
