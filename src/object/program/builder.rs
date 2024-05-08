use std::marker::PhantomData;

use super::shader;
use super::shader::prelude::*;
use super::shader::parameters;
use super::internal;
use super::uniform::Definitions;
use super::uniform::Uniforms;
use super::uniform;
use crate::glsl;
use crate::glsl::prelude::*;
use crate::glsl::location;
use crate::hlist;
use crate::hlist::lhlist;
use crate::object::shader::parameters::Parameters;



pub struct Data<IS, OS>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out>,
{
    inputs: PhantomData<IS>,
    outputs: PhantomData<OS>,
}

impl<IS, OS> Default for Data<IS, OS>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out>,
{
    fn default() -> Self {
        Self { inputs: Default::default(), outputs: Default::default() }
    }
}

pub struct Builder<'shaders, T, IS, OS, DUS, UUS>
where
    T: shader::target::Target,
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out>,
    DUS: uniform::marker::Definitions,
    UUS: uniform::marker::RDeclarations
{
    _target_phantom: PhantomData<T>,
    _data: Data<IS, OS>,
    uniforms: Uniforms<DUS, UUS>,
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
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out>,
    DUS: uniform::marker::Definitions,
{
    // TODO: clean this up
    fn retype_attach<NT, NOS, NUS>(self) -> Builder<'s, NT, IS, NOS, DUS, NUS>
    where
        NT: shader::target::Target,
        NOS: parameters::Parameters<Out>,
        NUS: uniform::marker::RDeclarations
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

    fn retype_vertex_attach<NIS, NOS, NUS>(self) -> Builder<'s, Vertex, NIS, NOS, DUS, NUS>
    where
        NIS: parameters::Parameters<In>,
        NOS: parameters::Parameters<Out>,
        NUS: uniform::marker::RDeclarations
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

    fn retype_unmatched_uniforms<NUUS>(self, uniforms: Uniforms<DUS, NUUS>) -> Builder<'s, Vertex, IS, OS, DUS, NUUS>
    where
        NUUS: uniform::marker::RDeclarations
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

    fn retype_definitions<NDUS>(self, definitions: Definitions<NDUS>) -> Builder<'s, Vertex, (), (), NDUS, ()>
    where
        NDUS: uniform::marker::Definitions
    {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            uniforms: Uniforms::new(definitions),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    // 3 kinds of API
    // uniform_xyz_default -- assigns default values on creation
    // uniform_xyz_initializer -- assigns values from registered callbacks
    // uniform_xyz -- expects parameter in program creation
    // just build list in builder?
}

impl<'s, T, IS, OS, DUS, HUUS, TUUS, const LOCATION: usize> Builder<'s, T, IS, OS, DUS, (uniform::Declaration<HUUS, LOCATION>, TUUS)>
where
    T: shader::target::Target,
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out>,
    DUS: uniform::marker::Definitions,
    HUUS: glsl::Uniform,
    TUUS: uniform::marker::RDeclarations,
    (uniform::Declaration<HUUS, LOCATION>, TUUS): uniform::marker::RDeclarations
{
    pub fn bind_uniforms(self, matcher: impl FnOnce(Uniforms<DUS, (uniform::Declaration<HUUS, LOCATION>, TUUS)>) -> Uniforms::<DUS, ()>) -> Builder<'s, T, IS, OS, DUS, ()> {
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
            compute: Default::default()
        }
    }
}

impl<'s> Builder<'s, Vertex, (), (), (), ()> {
    pub fn new() -> Self {
        Self::default()
    }
   
    pub fn uniforms<DUS>(self, definer: impl FnOnce(Definitions<()>) -> Definitions<DUS>) -> Builder<'s, Vertex, (), (), DUS, ()>
    where
        DUS: uniform::marker::Definitions,
    {
        let definitions = definer(Definitions::new());
        self.retype_definitions(definitions)
    }
}

impl<'s, DUS> Builder<'s, Vertex, (), (), DUS, ()>
where
    DUS: uniform::marker::Definitions,
{
    pub fn vertex_main<VI, VO, US>(mut self, vertex: &'s super::Main<Vertex, VI, VO, US>) -> Builder<Vertex, VI, VO, DUS, US::Inverted>
    where
        VI: super::parameters::Parameters<In>,
        VO: super::parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations
    {
        self.vertex = Some(internal::ShaderStage::new(&vertex.0));
        self.retype_vertex_attach()
    }
}

/// impl for vertex stage
impl<'s, IS, OS, DUS> Builder<'s, Vertex, IS, OS, DUS, ()>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions,
{
    /// Attach new vertex shader for linking purposes possibly adding new uniforms.
    pub fn vertex_shared<US>(mut self, vertex: &'s Shared<Vertex, US>) -> Builder<'_, Vertex, IS, OS, DUS, US::Inverted>
    where
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.vertex.as_mut().expect("vertex stage is set").shared.push(&vertex.0);
        let declarations = self.uniforms.clone().add_unmatched();
        self.retype_unmatched_uniforms(declarations)
    }

    pub fn tesselation_control_main<TCO, US>(mut self, tesselation_control: &'s Main<tesselation::Control, OS::Inputs, TCO, US>) -> Builder<tesselation::Control, IS, TCO, DUS, US::Inverted>
    where
        TCO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.tesselation_control = Some(internal::ShaderStage::new(&tesselation_control.0));
        self.retype_attach()
    }

    pub fn geometry_main<GO, US>(mut self, geometry: &'s Main<Geometry, OS::Inputs, GO, US>) -> Builder<Geometry, IS, GO, DUS, US::Inverted>
    where
        GO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.geometry = Some(internal::ShaderStage::new(&geometry.0));
        self.retype_attach()
    }

    pub fn fragment_main<FO, US>(mut self, fragment: &'s Main<Fragment, OS::Inputs, FO, US>) -> Builder<Fragment, IS, FO, DUS, US::Inverted>
    where
        FO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.fragment.replace(internal::ShaderStage::new(&fragment.0));
        self.retype_attach()
    }
}

/// impl for tesselation control stage
impl<'s, IS, OS, DUS> Builder<'s, tesselation::Control, IS, OS, DUS, ()>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions,
{
    pub fn tesselation_control_shared<US>(mut self, tesselation_control: &'s Shared<tesselation::Control, US>) -> Builder<'_, tesselation::Control, IS, OS, DUS, US::Inverted>
    where
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.tesselation_control.as_mut().expect("tesselation control was initialized").shared.push(&tesselation_control.0);
        self.retype_attach()
    }

    pub fn tesselation_evaluation_main<TEO, US>(mut self, tesselation_evaluation: &'s Main<tesselation::Evaluation, OS::Inputs, TEO, US>) -> Builder<tesselation::Evaluation, IS, TEO, DUS, US::Inverted>
    where
        TEO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {    
        self.tesselation_evaluation = Some(internal::ShaderStage::new(&tesselation_evaluation.0));
        self.retype_attach()
    }
}

/// impl for tesselation evaluation stage
impl<'s, IS, OS, DUS> Builder<'s, tesselation::Evaluation, IS, OS, DUS, ()>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions
{
    pub fn tesselation_evaluation_shared<US>(mut self, shared: &'s Shared<tesselation::Evaluation, US>) -> Builder<'_, tesselation::Evaluation, IS, OS, DUS, US::Inverted>
    where
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.tesselation_evaluation.as_mut().expect("tesselation evaluation stage was initialized").shared.push(&shared.0);
        self.retype_attach()
    }

    pub fn geometry_main<GO, US>(mut self, geometry: &'s Main<Geometry, OS::Inputs, GO, US>) -> Builder<Geometry, IS, GO, DUS, US::Inverted>
    where
        GO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.geometry = Some(internal::ShaderStage::new(&geometry.0));
        self.retype_attach()
    }

    pub fn fragment_main<FO, US>(mut self, fragment: &'s Main<Fragment, OS::Inputs, FO, US>) -> Builder<Fragment, IS, FO, DUS, US::Inverted>
    where
        FO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.fragment = Some(internal::ShaderStage::new(&fragment.0));
        self.retype_attach()
    }
}

/// impl for geometry stage
impl<'s, IS, OS, DUS> Builder<'s, Geometry, IS, OS, DUS, ()>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out> + MatchingInputs,
    DUS: uniform::marker::Definitions
{
    pub fn geometry_shared<US>(mut self, geometry: &'s Shared<Geometry, US>) -> Builder<'_, Geometry, IS, OS, DUS, US::Inverted>
    where
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.geometry.as_mut().expect("geometry stage was initialized").shared.push(&geometry.0);
        self.retype_attach()
    }

    pub fn fragment_main<FO, US>(mut self, fragment: &'s Main<Fragment, OS::Inputs, FO, US>) -> Builder<Fragment, IS, FO, DUS, US::Inverted>
    where
        FO: parameters::Parameters<Out>,
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.fragment = Some(internal::ShaderStage::new(&fragment.0));
        self.retype_attach()
    }
}

/// impl for fragment stage
impl<'s, IS, OS, DUS> Builder<'s, Fragment, IS, OS, DUS, ()>
where
    IS: parameters::Parameters<In>,
    OS: parameters::Parameters<Out>,
    DUS: uniform::marker::Definitions
{
    pub fn fragment_shared<US>(mut self, fragment: &'s Shared<Fragment, US>) -> Builder<'_, Fragment, IS, OS, DUS, US::Inverted>
    where
        US: uniform::marker::LDeclarations + hlist::lhlist::Invert,
        US::Inverted: uniform::marker::RDeclarations,
    {
        self.fragment.as_mut().expect("fragment stage was initialized").shared.push(&fragment.0);
        self.retype_attach()
    }

    pub fn build(&self) -> Result<super::Program<IS, OS, DUS>, super::LinkingError> {

        let program = super::Program::create_with_uniforms(self.uniforms.clone());

        program.attach(self.vertex.as_ref().expect("vertex shader stage is set"));

        if let (Some(control_stage), Some(evaluation_stage)) = (&self.tesselation_control, &self.tesselation_evaluation) {
            program.attach(control_stage);
            program.attach(evaluation_stage);
        }

        if let Some(geometry) = &self.geometry {
            program.attach(geometry);
        }

        program.attach(self.fragment.as_ref().expect("fragment shader stage is set"));
        program.link()
    }
}


// pub mod attach {
//     use crate::object::shader::TargetProvider;

//     use super::{parameters, Uniforms, Builder};
//     use super::shader;
//     use super::shader::prelude::*;
//     use super::Data;


//     /// Relation that determines next type after attaching
//     pub(crate) trait NextTarget {
//         type Next: Target;
//     }

//     macro_rules! impl_next_target {
//         ($what: ty, $for: ty => $next: ty) => {
//             impl NextTarget for ($what, $for) {
//                 type Next = $next;
//             }
//         };
//         ($what: path, $for: path => $next: path) => {
//             impl NextTarget for ($what, $for) {
//                 type Next = $next;
//             }
//         };
//     }

//     impl_next_target! { tesselation::Control, Vertex => tesselation::Control }
//     impl_next_target! { Geometry , Vertex => Geometry }
//     impl_next_target! { Fragment , Vertex => Fragment }

//     impl_next_target! { tesselation::Evaluation, tesselation::Control => tesselation::Evaluation }

//     impl_next_target! { Geometry , tesselation::Evaluation => Geometry }
//     impl_next_target! { Fragment , tesselation::Evaluation => Fragment }

//     impl_next_target! { Fragment , Geometry => Fragment }

//     impl<'s, T, IS, OS, DUS, US> shader::TargetProvider for Builder<'s, T, IS, OS, DUS, US>
//     where
//         T: Target,
//         IS: parameters::Parameters,
//         OS: parameters::Parameters,
//         DUS: crate::hlist::lhlist::Append,
//     {
//         type Target = T;
//     }

//     impl<T, IS, OS, US> shader::TargetProvider for Main<T, IS, OS, US>
//     where
//         T: Target,
//         IS: parameters::Parameters,
//         OS: parameters::Parameters,
//     {
//         type Target = T;
//     }

//     impl<T, US> shader::TargetProvider for Shared<T, US>
//     where
//         T: shader::target::Target,
//     {
//         type Target = T;
//     }

//     /// Allows for specialization of attach based on type.
//     trait GenericAttach<'s> {
//         type Shader: shader::target::Target;
//         type Result<US>;

//         fn generic_attach<US>(&mut self, shader: &'s shader::internal::CompiledShader<Self::Shader>) -> Self::Result<US>;
//     }

//     macro_rules! impl_generic_attach {
//         ($target: ty, $accessor: ident) => {
//             impl<'s, IS, OS, DUS> GenericAttach<'s> for Builder<'s, $target, IS, OS, DUS, ()>
//             where
//                 IS: parameters::Parameters,
//                 OS: parameters::Parameters,
//                 DUS: crate::hlist::lhlist::Append,
//             {
//                 type Shader = <Self as shader::TargetProvider>::Target;
//                 type Result<US> = Builder<'s, $target, IS, OS, DUS, US>;

//                 fn generic_attach<US>(&mut self, shader: &'s shader::internal::CompiledShader<Self::Shader>) -> Self::Result<US> {
//                     match &mut self.$accessor {
//                         Some(stage) => stage.shared.push(shader),
//                         None => self.$accessor = Some(super::internal::ShaderStage::new::<US>(shader)),
//                     };
//                     self.retype_attach()
//                 }
//             }
//         };
//     }

//     impl<'s, IS, OS, DUS> GenericAttach<'s> for Builder<'s, Vertex, IS, OS, DUS, ()>
//     where
//         IS: parameters::Parameters,
//         OS: parameters::Parameters,
//         DUS: crate::hlist::lhlist::Append,
//     {
//         type Shader = <Self as shader::TargetProvider>::Target;
//         type Result<US> = Builder<'s, Vertex, IS, OS, DUS, US>;

//         fn generic_attach<US>(&mut self, shader: &'s shader::internal::CompiledShader<Self::Shader>) -> Self::Result<US> {
//             self.vertex.shared.push(shader);
//             self.retype_unmatched_uniforms(self.uniforms.add_unmatched())
//         }
//     }

//     impl_generic_attach! { tesselation::Control, tesselation_control }
//     impl_generic_attach! { tesselation::Evaluation, tesselation_evaluation }
//     impl_generic_attach! { Geometry, geometry }
//     impl_generic_attach! { Fragment, fragment }
//     impl_generic_attach! { Compute, compute }
//     //endregion

//     //region [ rgba(38, 60, 72, 0.1) ] Attach
//     /// Relation on types that represent possible attachments for program with given last shader
//     pub(crate) trait AttachShared<'s, T>
//     where
//         T: Target,
//         T: 's, 
//     {
//         type Result<US> where US: 's;

//         /// Attach new shared shader
//         fn attach<US>(self, shader: &'s Shared<T, US>) -> Self::Result<US>;
//     }

//     impl<'s, T, IS, OS, DUS> AttachShared<'s, T> for Builder<'s, T, IS, OS, DUS, ()>
//     where
//         Self: GenericAttach<'s, Shader = T>,
//         T: Target + 's,
//         IS: parameters::Parameters,
//         OS: parameters::Parameters,
//         DUS: crate::hlist::lhlist::Append,
//     {
//         type Result<US: 's> = Builder<'s, T, IS, OS, DUS, US>;

//         fn attach<US>(mut self, shader: &'s Shared<T, US>) -> Self::Result<US> {
//             <Self as GenericAttach>::generic_attach::<US>(&mut self, &shader.0);
//             self.retype_attach()
//         }
//     }

//     /// Relation on types that represent possible attachments for program with given last shader
//     pub(crate) trait AttachMain<'s, T>: TargetProvider
//     where
//         // TODO: understand the role of lifetime bonds here
//         T: shader::TargetProvider + 's,
//         Self: shader::TargetProvider,
//         (T::Target, Self::Target): NextTarget,
//         (Self::NextTarget, <Self as TargetProvider>::Target): NextTarget,
//     {
//         type NextTarget;
//         type Output<NO, US>: shader::TargetProvider<Target = Self::NextTarget> 
//         where 
//             NO: parameters::Parameters,
//             US: 's,
//         ;

//         /// Attach new shader
//         fn attach<NO, US>(self, shader: &'s T) -> Self::Output<NO, US>
//         where
//             NO: parameters::Parameters,
//             US: 's,
//         ;
//     }

//     impl<'s, CT, NT, CI, CO, DUS> AttachMain<'s, NT> for Builder<'s, CT, CI, CO, DUS, ()>
//     where
//         CT: Target,
//         NT: Target,
//         CI: parameters::Parameters,
//         CO: parameters::Parameters,
//         CT: shader::TargetProvider,
//         NT: shader::TargetProvider,
//         Self: shader::TargetProvider,
//         Self: GenericAttach<'s, Shader = NT>,
//         DUS: crate::hlist::lhlist::Append,
//     {
//         type Output<NO, US> = Builder<'s, (), CI, NO, DUS, US>
//         where
//             NO: parameters::Parameters,
//             US: 's
//         ;

//         fn attach<NO, US>(mut self, shader: &'s Main<NT, CO, NO, US>) -> Self::Output<NO, US>
//         where
//             NO: parameters::Parameters,
//             US: 's,
//         {
//             <Self as GenericAttach>::generic_attach::<US>(&mut self, &shader.0);
//             Builder {
//                 _target_phantom: std::marker::PhantomData,
//                 uniforms: self.uniforms.add_,
//                 _data: super::Data::default(),
//                 vertex: self.vertex,
//                 tesselation_control: self.tesselation_control,
//                 tesselation_evaluation: self.tesselation_evaluation,
//                 geometry: self.geometry,
//                 fragment: self.fragment,
//                 compute: self.compute,
//             }
//         }
//     }

//     mod older {
//         // impl<'s, CT: Target, NT: Target, CI: Type, CO: Type, NO: Type> AttachMain<'s, Main<NT, CO, NO>> for Builder<'s, CT, CI, CO>
//         // where
//         //     CT: TargetProvider,
//         //     NT: TargetProvider,
//         //     Self: TargetProvider,
//         //     (NT, Self::Target): NextTarget,
//         //     Self: GenericAttach<'s, Shader = NT>,
//         // {
//         //     type Output = Builder<'s, <(NT, Self::Target) as NextTarget>::Next, CI, NO>;

//         //     fn attach(mut self, shader: &'s Main<NT, CO, NO>) -> Self::Output {
//         //         <Self as GenericAttach>::generic_attach(&mut self, &shader.0);
//         //         Builder {
//         //             _target_phantom: std::marker::PhantomData,
//         //             _input_phantom: std::marker::PhantomData,
//         //             _output_phantom: std::marker::PhantomData,
//         //             vertex: self.vertex,
//         //             tesselation_control: self.tesselation_control,
//         //             tesselation_evaluation: self.tesselation_evaluation,
//         //             geometry: self.geometry,
//         //             fragment: self.fragment,
//         //             compute: self.compute,
//         //         }
//         //     }
//         // }
//     }

//     //endregion

//     // impl<'s, OldI: Type, OldO: Type, NewI: Type, NewO: Type> Attach<'s, Main<Vertex, NewI, NewO>> for Builder<'s, Vertex, OldI, OldO> {
//     //     type Updated = Self;

//     //     fn attach(self, shader: &'s Main<Vertex, NewI, NewO>) -> Self {
//     //         todo!()
//     //     }
//     // }

//     // impl<'s, I: Type, VO: Type> Attach<'s, tesselation::Control> for Builder<'s, Vertex, I, VO> {
//     //     type Result = Builder<'s, tesselation::Control, VO, TCO>;

//     //     fn attach(self, shader: &'s TesselationControlShader) -> Self::Result {
//     //         self.tesselation_control.push(shader);
//     //         self
//     //     }
//     // }

//     // impl Attach<Geometry> for Vertex {
//     //     type Result = Geometry;

//     //     fn attach(self, shader: Geometry) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // impl Attach<Fragment> for Vertex {
//     //     type Result = Fragment;

//     //     fn attach(self, shader: Fragment) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // // Tesselation attachments
//     // impl Attach<tesselation::Control> for tesselation::Control {
//     //     type Result = Self;

//     //     fn attach(self, shader: tesselation::Control) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // impl Attach<tesselation::Evaluation> for tesselation::Control {
//     //     type Result = tesselation::Evaluation;

//     //     fn attach(self, shader: tesselation::Control) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // impl Attach<tesselation::Evaluation> for tesselation::Evaluation {
//     //     type Result = Self;

//     //     fn attach(self, shader: tesselation::Evaluation) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // impl Attach<Geometry> for tesselation::Evaluation {
//     //     type Result = Geometry;

//     //     fn attach(self, shader: Geometry) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // impl Attach<Fragment> for tesselation::Evaluation {
//     //     type Result = Fragment;

//     //     fn attach(self, shader: Fragment) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // // Geometry attachment
//     // impl Attach<Geometry> for Geometry {
//     //     type Result = Self;

//     //     fn attach(self, shader: Geometry) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // impl Attach<Fragment> for Geometry {
//     //     type Result = Fragment;

//     //     fn attach(self, shader: Fragment) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

//     // // Fragment attachment
//     // impl Attach<Fragment> for Fragment {
//     //     type Result = Self;

//     //     fn attach(self, shader: Fragment) -> Self::Result {
//     //         todo!()
//     //     }
//     // }

// }
