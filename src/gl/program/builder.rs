
use crate::prelude::internal::*;

// root imports
use crate::gl;
use crate::glsl;
use crate::glsl::variable;
use crate::hlist;
use crate::hlist::lhlist;
use crate::ts;
use crate::utils;

// sub imports
use gl::shader;
use gl::uniform;

// item imports
use variable::storage::{In, Out};
use gl::uniform::{Definitions, Matcher};
use gl::object::ObjectBase;
use gl::shader::*;
use gl::program::ShaderStage;

use target::*;

use super::Resources;

pub struct Params<Ins, Outs>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
{
    inputs: PhantomData<Ins>,
    outputs: PhantomData<Outs>,
}

impl<Ins, Outs> Default for Params<Ins, Outs>
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

pub trait MaybeTarget: shader::Target + ts::Maybe { }

mod maybe {
    use super::*;

    impl gl::target::Target for ts::None {
        const ID: u32 = panic!("`None` does not implement `Target`");
    }
    hi::denmark! { ts::None as MaybeTarget, shader::Target }

    impl<T> gl::target::Target for ts::Some<T> where T: Target {
        const ID: u32 = T::ID;
    }
    impl<T: Target> shader::Target for ts::Some<T> { }
    impl<T: Target> MaybeTarget for ts::Some<T> { }
}

pub struct Builder<'shaders, Target, Ins, Outs, Defs, Decls, Res>
where
    Target: MaybeTarget,
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
    Decls: uniform::bounds::Declarations,
{
    _target_phantom: PhantomData<Target>,
    _parameters: Params<Ins, Outs>,
    resource_phantoms: Resources<Res>,
    matcher: Option<uniform::Matcher<Defs, Decls>>,
    vertex: Option<ShaderStage<'shaders, Vertex>>,
    tess_control: Option<ShaderStage<'shaders, TessControl>>,
    tess_evaluation: Option<ShaderStage<'shaders, TessEvaluation>>,
    geometry: Option<ShaderStage<'shaders, Geometry>>,
    fragment: Option<ShaderStage<'shaders, Fragment>>,
    compute: Option<ShaderStage<'shaders, Compute>>,
}

impl<'s> Default for Builder<'s, ts::None, (), (), (), (), ()> {
    fn default() -> Self {
        Self { 
            _target_phantom: Default::default(),
            _parameters: Default::default(),
            resource_phantoms: Default::default(),
            matcher: Default::default(),
            vertex: Default::default(),
            tess_control: Default::default(),
            tess_evaluation: Default::default(),
            geometry: Default::default(),
            fragment: Default::default(),
            compute: Default::default(),
        }
    }
}

impl<'s> Builder<'s, ts::None, (), (), (), (), ()> {
    /// Create empty Builder.
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn no_uniforms(self) -> Self {
        self.uniforms(std::convert::identity)
    }

    /// Define uniforms and resources that program uses.
    pub fn uniforms<Defs: uniform::bounds::Definitions>(self, provider: impl FnOnce(uniform::Definitions<()>) -> uniform::Definitions<Defs>) -> Builder<'s, ts::None, (), (), Defs, (), ()> {
        let definitions = provider(uniform::Definitions::default());
        Builder {
            _target_phantom: PhantomData,
            _parameters: self._parameters,
            matcher: Some(uniform::Matcher::new(definitions)),
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms: self.resource_phantoms,
        }
    }

    pub fn no_resources(self) -> Builder<'s, ts::Some<Vertex>, (), (), (), (), ()> {
        self.resources(std::convert::identity)
    }

    /// Define uniforms and resources that program uses.
    pub fn resources<Res>(self, provider: impl FnOnce(Resources<()>) -> Resources<Res>) -> Builder<'s, ts::Some<Vertex>, (), (), (), (), Res> {
        let resource_phantoms = provider(gl::program::Resources::default());
        Builder {
            _target_phantom: PhantomData,
            _parameters: self._parameters,
            matcher: self.matcher,
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms,
        }
    } 
}

impl<'s, DH, DT> Builder<'s, ts::None, (), (), (DH, DT), (), ()>
where
(DH, DT): uniform::bounds::Definitions
{
    pub fn no_resources(self) -> Builder<'s, ts::Some<Vertex>, (), (), (DH, DT), (), ()> {
        self.resources(std::convert::identity)
    }

    /// Define uniforms and resources that program uses.
    pub fn resources<Res>(self, provider: impl FnOnce(Resources<()>) -> Resources<Res>) -> Builder<'s, ts::Some<Vertex>, (), (), (DH, DT), (), Res> {
        let resource_phantoms = provider(gl::program::Resources::default());
        Builder {
            _target_phantom: PhantomData,
            _parameters: self._parameters,
            matcher: self.matcher,
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms,
        }
    } 
}

impl<'s, Target, Ins, Outs, Defs, Res> Builder<'s, ts::Some<Target>, Ins, Outs, Defs, (), Res>
where
    Target: shader::Target,
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
{
    /// Update type parameters on `Main` shader attachment.
    /// 
    /// `Main` shader attachment advances Builder's `Target`, `Outs` and `Decls` parameters.
    fn attach_main<NTarget, NOuts, Decls>(self, decls: uniform::Declarations<ts::Mutable, Decls>) -> Builder<'s, ts::Some<NTarget>, Ins, NOuts, Defs, Decls, Res>
    where
        NTarget: shader::Target,
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _parameters: Default::default(),
            matcher: self.matcher.map(|inner|inner.set_declarations(decls)),
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms: self.resource_phantoms,
        }
    }

    /// Update type parameters on `Lib` shader attachment.
    /// 
    /// `Shared` shader can require some additional uniforms.
    fn attach_lib<Decls>(self, decls: uniform::Declarations<ts::Mutable, Decls>) -> Builder<'s, ts::Some<Vertex>, Ins, Outs, Defs, Decls, Res>
    where
        Decls: uniform::bounds::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _parameters: self._parameters,
            matcher: self.matcher.map(|inner|inner.set_declarations(decls)),
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms: self.resource_phantoms,
        }
    }

    /// ts::Some<Vertex> shader attachment is different as it also sets `Ins` (from initially empty list).
    fn attach_vertex_main<NIns, NOuts, Decls>(self, decls: uniform::Declarations<ts::Mutable, Decls>) -> Builder<'s, ts::Some<Vertex>, NIns, NOuts, Defs, Decls, Res>
    where
        NIns: glsl::Parameters<In>,
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        Builder {
            _target_phantom: PhantomData,
            _parameters: Default::default(),
            matcher: self.matcher.map(|inner|inner.set_declarations(decls)),
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms: self.resource_phantoms,
        }
    }
}

impl<'s, Target, Ins, Outs, Defs, Decls, Res> Builder<'s, ts::Some<Target>, Ins, Outs, Defs, Decls, Res>
where
    Target: shader::Target,
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
    Decls: uniform::bounds::Declarations,
{
    /// Map uniform declarations from most recently attached shader to definitions provided by the program. 
    pub fn uniforms(self, matcher: impl FnOnce(Matcher<Defs, Decls>) -> Matcher<Defs, ()>) -> Builder<'s, ts::Some<Target>, Ins, Outs, Defs, (), Res> {
        Builder {
            _target_phantom: PhantomData,
            _parameters: self._parameters,
            matcher: self.matcher.map(matcher),
            vertex: self.vertex,
            tess_control: self.tess_control,
            tess_evaluation: self.tess_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            resource_phantoms: self.resource_phantoms,
        }
    }
}

/// impl for initial stage
impl<'s, Defs, Res> Builder<'s, ts::Some<Vertex>, (), (), Defs, (), Res>
where
    Defs: uniform::bounds::Definitions,
{
    pub fn vertex_main<VIns, VOuts, Decls>(mut self, vertex: &'s super::Main<Vertex, VIns, VOuts, Decls>) -> Builder<ts::Some<Vertex>, VIns, VOuts, Defs, Decls, Res>
    where
        VIns: super::glsl::Parameters<In>,
        VOuts: super::glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.vertex = Some(ShaderStage::new(&vertex));
        self.attach_vertex_main(vertex.declarations())
    }
}

/// impl for vertex stage
impl<'s, Ins, Outs, Defs, Res> Builder<'s, ts::Some<Vertex>, Ins, Outs, Defs, (), Res>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + glsl::variable::MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    /// Attach new vertex shader for linking purposes possibly adding new uniforms.
    pub fn vertex_shared<Decls>(mut self, vertex: &'s Lib<Vertex, Decls>) -> Builder<'_, ts::Some<Vertex>, Ins, Outs, Defs, Decls, Res>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.vertex
            .as_mut()
            .expect("vertex stage is set")
            .libs
            .push(vertex);
        self.attach_lib(vertex.declarations())
    }

    pub fn tess_control_main<NOuts, Decls>(mut self, tess_control: &'s Main<TessControl, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<TessControl>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.tess_control = Some(ShaderStage::new(&tess_control.0));
        self.attach_main(tess_control.declarations())
    }

    pub fn geometry_main<NOuts, Decls>(mut self, geometry: &'s Main<Geometry, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<Geometry>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.geometry = Some(ShaderStage::new(&geometry.0));
        self.attach_main(geometry.declarations())
    }

    pub fn fragment_main<NOuts, Decls>(mut self, fragment: &'s Main<Fragment, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<Fragment>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.fragment
            .replace(ShaderStage::new(&fragment.0));
        self.attach_main(fragment.declarations())
    }
}

/// impl for tesselation control stage
impl<'s, Ins, Outs, Defs, Res> Builder<'s, ts::Some<TessControl>, Ins, Outs, Defs, (), Res>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + glsl::MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    pub fn tess_control_shared<Decls>(mut self, tess_control: &'s Lib<TessControl, Decls>) -> Builder<ts::Some<TessControl>, Ins, Outs, Defs, Decls, Res>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.tess_control
            .as_mut()
            .expect("tesselation control was initialized")
            .libs
            .push(tess_control);
        self.attach_main(tess_control.declarations())
    }

    pub fn tess_evaluation_main<NOuts, Decls>(mut self, tess_evaluation_main: &'s Main<TessEvaluation, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<TessEvaluation>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.tess_evaluation = Some(ShaderStage::new(tess_evaluation_main));
        self.attach_main(tess_evaluation_main.declarations())
    }
}

/// impl for tesselation evaluation stage
impl<'s, Ins, Outs, Defs, Res> Builder<'s, ts::Some<TessEvaluation>, Ins, Outs, Defs, (), Res>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + glsl::MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    pub fn tesselation_evaluation_shared<Decls>(mut self, te_lib: &'s Lib<TessEvaluation, Decls>) -> Builder<ts::Some<TessEvaluation>, Ins, Outs, Defs, Decls, Res>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.tess_evaluation
            .as_mut()
            .expect("tesselation evaluation stage was initialized")
            .libs
            .push(te_lib);
        self.attach_main(te_lib.declarations())
    }

    pub fn geometry_main<NOuts, Decls>(mut self, geometry: &'s Main<Geometry, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<Geometry>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.geometry = Some(ShaderStage::new(geometry));
        self.attach_main(geometry.declarations())
    }

    pub fn fragment_main<NOuts, Decls>(mut self, fragment: &'s Main<Fragment, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<Fragment>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.fragment = Some(ShaderStage::new(&fragment.0));
        self.attach_main(fragment.declarations())
    }
}

/// impl for geometry stage
impl<'s, Ins, Outs, Defs, Res> Builder<'s, ts::Some<Geometry>, Ins, Outs, Defs, (), Res>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out> + glsl::MatchingInputs,
    Defs: uniform::bounds::Definitions,
{
    pub fn geometry_shared<Decls>(mut self, geometry: &'s Lib<Geometry, Decls>) -> Builder<ts::Some<Geometry>, Ins, Outs, Defs, Decls, Res>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.geometry
            .as_mut()
            .expect("geometry stage was initialized")
            .libs
            .push(geometry);
        self.attach_main(geometry.declarations())
    }

    pub fn fragment_main<NOuts, Decls>(mut self, fragment: &'s Main<Fragment, Outs::Inputs, NOuts, Decls>) -> Builder<ts::Some<Fragment>, Ins, NOuts, Defs, Decls, Res>
    where
        NOuts: glsl::Parameters<Out>,
        Decls: uniform::bounds::Declarations,
    {
        self.fragment = Some(ShaderStage::new(&fragment.0));
        self.attach_main(fragment.declarations())
    }
}

/// impl for fragment stage
impl<'s, Ins, Outs, Defs, Res> Builder<'s, ts::Some<Fragment>, Ins, Outs, Defs, (), Res>
where
    Ins: glsl::Parameters<In>,
    Outs: glsl::Parameters<Out>,
    Defs: uniform::bounds::Definitions,
{
    pub fn fragment_shared<Decls>(mut self, fragment: &'s Lib<Fragment, Decls>) -> Builder<ts::Some<Fragment>, Ins, Outs, Defs, Decls, Res>
    where
        Decls: uniform::bounds::Declarations,
    {
        self.fragment
            .as_mut()
            .expect("fragment stage was initialized")
            .libs
            .push(fragment);
        self.attach_main(fragment.declarations())
    }

    /// Build `Program` by linking all the provided attachments.
    pub fn build(&self) -> Result<super::Program<Ins, Outs, Defs::AsDeclarations, Res>, super::LinkingError> {
        let program = super::Program::create_with_uniforms(&self.matcher.as_ref().expect("matcher is provided").definitions);

        program.attach(self.vertex.as_ref().expect("vertex shader stage is set"));

        if let (Some(control_stage), Some(evaluation_stage)) =
            (&self.tess_control, &self.tess_evaluation)
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
