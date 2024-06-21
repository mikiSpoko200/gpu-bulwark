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
use crate::utils;

pub struct Data<IS, OS>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
{
    inputs: PhantomData<IS>,
    outputs: PhantomData<OS>,
}

impl<IS, OS> Default for Data<IS, OS>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
{
    fn default() -> Self {
        Self {
            inputs: Default::default(),
            outputs: Default::default(),
        }
    }
}

pub struct Builder<'shaders, T, IS, OS, DUS, UUS>
where
    T: shader::target::Target,
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
    DUS: uniform::marker::Definitions,
    UUS: uniform::marker::Declarations,
{
    _target_phantom: PhantomData<T>,
    _data: Data<IS, OS>,
    uniforms: uniform::Matcher<DUS, UUS>,
    vertex: Option<internal::ShaderStage<'shaders, Vertex>>,
    tesselation_control: Option<internal::ShaderStage<'shaders, tesselation::Control>>,
    tesselation_evaluation: Option<internal::ShaderStage<'shaders, tesselation::Evaluation>>,
    geometry: Option<internal::ShaderStage<'shaders, Geometry>>,
    fragment: Option<internal::ShaderStage<'shaders, Fragment>>,
    compute: Option<internal::ShaderStage<'shaders, Compute>>,
}

impl<'s, T, IS, OS, DUS> Builder<'s, T, IS, OS, DUS, ()>
where
    T: shader::target::Target,
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
    DUS: uniform::marker::Definitions,
{
    // TODO: clean this up
    fn retype_attach<NT, NOS, US>(self) -> Builder<'s, NT, IS, NOS, DUS, US>
    where
        NT: shader::target::Target,
        NOS: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            uniforms: self.uniforms.add_unmatched(),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    fn retype_vertex_attach<NIS, NOS, US>(self) -> Builder<'s, Vertex, NIS, NOS, DUS, US>
    where
        NIS: glsl::Parameters<In>,
        NOS: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            uniforms: self.uniforms.add_unmatched(),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    /// Retype only unmatched uniforms
    fn retype_only_unmatched_uniforms<US>(self, uniforms: Matcher<DUS, US>) -> Builder<'s, Vertex, IS, OS, DUS, US>
    where
        US: uniform::marker::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: self._data,
            uniforms: uniforms,
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    fn retype_definitions<US>(self, definitions: US) -> Builder<'s, Vertex, (), (), US, ()>
    where
        US: uniform::marker::Definitions,
    {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            uniforms: Matcher::new(definitions),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }
}

impl<'s, T, IS, OS, DUS, UUS> Builder<'s, T, IS, OS, DUS, UUS>
where
    T: shader::target::Target,
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
    DUS: uniform::marker::Definitions,
    UUS: uniform::marker::Declarations,
{
    pub fn bind_uniforms(self, matcher: impl FnOnce(Matcher<DUS, UUS>) -> Matcher<DUS, ()>) -> Builder<'s, T, IS, OS, DUS, ()> {
        let matched = matcher(self.uniforms);
        Builder {
            _target_phantom: PhantomData,
            _data: self._data,
            uniforms: matched,
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }
}

impl Default for Builder<'_, Vertex, (), (), (), ()> {
    fn default() -> Self {
        Self {
            _target_phantom: Default::default(),
            _data: Default::default(),
            uniforms: Default::default(),
            vertex: Default::default(),
            tesselation_control: Default::default(),
            tesselation_evaluation: Default::default(),
            geometry: Default::default(),
            fragment: Default::default(),
            compute: Default::default(),
        }
    }
}

impl<'s> Builder<'s, Vertex, (), (), (), ()> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create new Builder and provide uniform values
    pub fn with_uniforms<DUS>(self, uniform_definitions: DUS) -> Builder<'s, Vertex, (), (), DUS, ()>
    where
        DUS: uniform::marker::Definitions,
    {
        self.retype_definitions(uniform_definitions)
    }
}

/// impl for initial stage
impl<'s, DUS> Builder<'s, Vertex, (), (), DUS, ()>
where
    DUS: uniform::marker::Definitions,
{
    pub fn vertex_main<VI, VO, US>(
        mut self,
        vertex: &'s super::Main<Vertex, VI, VO, US>,
    ) -> Builder<Vertex, VI, VO, DUS, US>
    where
        VI: super::glsl::Parameters<In>,
        VO: super::glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.vertex = Some(internal::ShaderStage::new(&vertex.0));
        self.retype_vertex_attach()
    }
}

/// impl for vertex stage
impl<'s, IS, OS, DUS> Builder<'s, Vertex, IS, OS, DUS, ()>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions,
{
    /// Attach new vertex shader for linking purposes possibly adding new uniforms.
    pub fn vertex_shared<US>(mut self, vertex: &'s Shared<Vertex, US>) -> Builder<'_, Vertex, IS, OS, DUS, US>
    where
        US: uniform::marker::Declarations,
    {
        self.vertex
            .as_mut()
            .expect("vertex stage is set")
            .shared
            .push(&vertex.0);
        let declarations = self.uniforms.clone().add_unmatched();
        self.retype_only_unmatched_uniforms(declarations)
    }

    pub fn tesselation_control_main<TCO, US>(mut self, tesselation_control: &'s Main<tesselation::Control, OS::Inputs, TCO, US>) -> Builder<tesselation::Control, IS, TCO, DUS, US>
    where
        TCO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.tesselation_control = Some(internal::ShaderStage::new(&tesselation_control.0));
        self.retype_attach()
    }

    pub fn geometry_main<GO, US>(mut self, geometry: &'s Main<Geometry, OS::Inputs, GO, US>) -> Builder<Geometry, IS, GO, DUS, US>
    where
        GO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.geometry = Some(internal::ShaderStage::new(&geometry.0));
        self.retype_attach()
    }

    pub fn fragment_main<FO, US>(mut self, fragment: &'s Main<Fragment, OS::Inputs, FO, US>) -> Builder<Fragment, IS, FO, DUS, US>
    where
        FO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.fragment
            .replace(internal::ShaderStage::new(&fragment.0));
        self.retype_attach()
    }
}

/// impl for tesselation control stage
impl<'s, IS, OS, DUS> Builder<'s, tesselation::Control, IS, OS, DUS, ()>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions,
{
    pub fn tesselation_control_shared<US>(mut self, tesselation_control: &'s Shared<tesselation::Control, US>) -> Builder<'_, tesselation::Control, IS, OS, DUS, US>
    where
        US: uniform::marker::Declarations,
    {
        self.tesselation_control
            .as_mut()
            .expect("tesselation control was initialized")
            .shared
            .push(&tesselation_control.0);
        self.retype_attach()
    }

    pub fn tesselation_evaluation_main<TEO, US>(mut self, tesselation_evaluation: &'s Main<tesselation::Evaluation, OS::Inputs, TEO, US>) -> Builder<tesselation::Evaluation, IS, TEO, DUS, US>
    where
        TEO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.tesselation_evaluation = Some(internal::ShaderStage::new(&tesselation_evaluation.0));
        self.retype_attach()
    }
}

/// impl for tesselation evaluation stage
impl<'s, IS, OS, DUS> Builder<'s, tesselation::Evaluation, IS, OS, DUS, ()>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions,
{
    pub fn tesselation_evaluation_shared<US>(mut self, shared: &'s Shared<tesselation::Evaluation, US>) -> Builder<'_, tesselation::Evaluation, IS, OS, DUS, US>
    where
        US: uniform::marker::Declarations,
    {
        self.tesselation_evaluation
            .as_mut()
            .expect("tesselation evaluation stage was initialized")
            .shared
            .push(&shared.0);
        self.retype_attach()
    }

    pub fn geometry_main<GO, US>(mut self, geometry: &'s Main<Geometry, OS::Inputs, GO, US>) -> Builder<Geometry, IS, GO, DUS, US>
    where
        GO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.geometry = Some(internal::ShaderStage::new(&geometry.0));
        self.retype_attach()
    }

    pub fn fragment_main<FO, US>(mut self, fragment: &'s Main<Fragment, OS::Inputs, FO, US>) -> Builder<Fragment, IS, FO, DUS, US>
    where
        FO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.fragment = Some(internal::ShaderStage::new(&fragment.0));
        self.retype_attach()
    }
}

/// impl for geometry stage
impl<'s, IS, OS, DUS> Builder<'s, Geometry, IS, OS, DUS, ()>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions,
{
    pub fn geometry_shared<US>(mut self, geometry: &'s Shared<Geometry, US>) -> Builder<Geometry, IS, OS, DUS, US>
    where
        US: uniform::marker::Declarations,
    {
        self.geometry
            .as_mut()
            .expect("geometry stage was initialized")
            .shared
            .push(&geometry.0);
        self.retype_attach()
    }

    pub fn fragment_main<FO, US>(mut self, fragment: &'s Main<Fragment, OS::Inputs, FO, US>) -> Builder<Fragment, IS, FO, DUS, US>
    where
        FO: glsl::Parameters<Out>,
        US: uniform::marker::Declarations,
    {
        self.fragment = Some(internal::ShaderStage::new(&fragment.0));
        self.retype_attach()
    }
}

/// impl for fragment stage
impl<'s, IS, OS, DUS> Builder<'s, Fragment, IS, OS, DUS, ()>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
    DUS: uniform::marker::Definitions,
{
    pub fn fragment_shared<US>(mut self, fragment: &'s Shared<Fragment, US>) -> Builder<'_, Fragment, IS, OS, DUS, US>
    where
        US: uniform::marker::Declarations,
    {
        self.fragment
            .as_mut()
            .expect("fragment stage was initialized")
            .shared
            .push(&fragment.0);
        self.retype_attach()
    }

    pub fn build(&self) -> Result<super::Program<IS, OS, DUS>, super::LinkingError> {
        let program = super::Program::create_with_uniforms(self.uniforms.clone());

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
