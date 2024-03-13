use super::{parameters, Builder, CompiledShader};
use crate::{
    object::shader::{Main, Shared, TargetProvider},
    target::shader,
    target::shader::{tesselation, Compute, Fragment, Geometry, Vertex},
};

use shader::Target;


/// Relation that determines next type after attaching
pub(crate) trait NextTarget {
    type Next: Target;
}

macro_rules! impl_next_target {
    ($what: ty, $for: ty => $next: ty) => {
        impl NextTarget for ($what, $for) {
            type Next = $next;
        }
    };
    ($what: path, $for: path => $next: path) => {
        impl NextTarget for ($what, $for) {
            type Next = $next;
        }
    };
}

impl_next_target! { tesselation::Control, Vertex => tesselation::Control }
impl_next_target! { Geometry , Vertex => Geometry }
impl_next_target! { Fragment , Vertex => Fragment }

impl_next_target! { tesselation::Evaluation, tesselation::Control => tesselation::Evaluation }

impl_next_target! { Geometry , tesselation::Evaluation => Geometry }
impl_next_target! { Fragment , tesselation::Evaluation => Fragment }

impl_next_target! { Fragment , Geometry => Fragment }

impl<'s, T, IS, OS, DUS, US> TargetProvider for Builder<'s, T, IS, OS, DUS, US>
where
    T: Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append,
{
    type Target = T;
}

impl<T, IS, OS> TargetProvider for Main<T, IS, OS>
where
    T: Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    type Target = T;
}

impl<T> TargetProvider for Shared<T>
where
    T: shader::Target,
{
    type Target = T;
}

/// Allows for specialization of attach based on type.
trait GenericAttach<'s> {
    type Shader: Target;

    fn generic_attach(&mut self, shader: &'s CompiledShader<Self::Shader>);
}

macro_rules! impl_generic_attach {
    ($target: ty, $accessor: ident) => {
        impl<'s, IS, OS, DUS, UUS> GenericAttach<'s> for Builder<'s, $target, IS, OS, DUS, UUS>
        where
            IS: parameters::Parameters,
            OS: parameters::Parameters,
            DUS: crate::hlist::lhlist::Append,
        {
            type Shader = <Self as TargetProvider>::Target;

            fn generic_attach(&mut self, shader: &'s CompiledShader<Self::Shader>) {
                match &mut self.$accessor {
                    Some(stage) => stage.shared.push(shader),
                    None => self.$accessor = Some(super::ShaderStage::new(shader)),
                }
            }
        }
    };
    ($target: path, $accessor: ident) => {
        impl<'s, I: Type, O: Type> GenericAttach<'s> for Builder<'s, $target, I, O> {
            type Shader = <Self as TargetProvider>::Target;

            fn generic_attach(&mut self, shader: &'s CompiledShader<Self::Shader>) {
                match &mut self.$accessor {
                    Some(stage) => stage.shared.push(shader),
                    None => self.$accessor = Some(super::ShaderStage::new(shader)),
                }
            }
        }
    };
}

impl<'s, IS, OS, DUS> GenericAttach<'s> for Builder<'s, Vertex, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append,
{
    type Shader = <Self as TargetProvider>::Target;

    fn generic_attach(&mut self, shader: &'s CompiledShader<Self::Shader>) {
        self.vertex.shared.push(shader);
    }
}

impl_generic_attach! { tesselation::Control, tesselation_control }
impl_generic_attach! { tesselation::Evaluation, tesselation_evaluation }
impl_generic_attach! { Geometry, geometry }
impl_generic_attach! { Fragment, fragment }
impl_generic_attach! { Compute, compute }
//endregion

//region [ rgba(38, 60, 72, 0.1) ] Attach
/// Relation on types that represent possible attachments for program with given last shader
pub(crate) trait AttachShared<'s, T>
where
    T: Target,
{
    /// Attach new shared shader
    fn attach(self, shader: &'s Shared<T>) -> Self;
}

impl<'s, T, IS, OS, DUS> AttachShared<'s, T> for Builder<'s, T, IS, OS, DUS, ()>
where
    Self: GenericAttach<'s, Shader = T>,
    T: Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append,
{
    fn attach(mut self, shader: &'s Shared<T>) -> Self {
        <Self as GenericAttach>::generic_attach(&mut self, &shader.0);
        self
    }
}

/// Relation on types that represent possible attachments for program with given last shader
pub(crate) trait AttachMain<'s, T>
where
    T: TargetProvider,
    Self: TargetProvider,
    (T::Target, Self::Target): NextTarget,
{
    type Output: TargetProvider<Target = <(T::Target, Self::Target) as NextTarget>::Next>;

    /// Attach new shader
    fn attach(self, shader: &'s T) -> Self::Output;
}

impl<'s, CT, NT, CI, CO, NO, DUS> AttachMain<'s, Main<NT, CO, NO>> for Builder<'s, CT, CI, CO, DUS, ()>
where
    CT: Target,
    NT: Target,
    CI: parameters::Parameters,
    CO: parameters::Parameters,
    NO: parameters::Parameters,
    CT: TargetProvider,
    NT: TargetProvider,
    Self: TargetProvider,
    (NT, Self::Target): NextTarget,
    Self: GenericAttach<'s, Shader = NT>,
    DUS: crate::hlist::lhlist::Append,
{
    type Output = Builder<'s, <(NT, Self::Target) as NextTarget>::Next, CI, NO, DUS, ()>;

    fn attach(mut self, shader: &'s Main<NT, CO, NO>) -> Self::Output {
        <Self as GenericAttach>::generic_attach(&mut self, &shader.0);
        Builder {
            _target_phantom: std::marker::PhantomData,
            _data: super::Data::default(),
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
            uniforms: todo!(),
        }
    }
}

mod older {
    // impl<'s, CT: Target, NT: Target, CI: Type, CO: Type, NO: Type> AttachMain<'s, Main<NT, CO, NO>> for Builder<'s, CT, CI, CO>
    // where
    //     CT: TargetProvider,
    //     NT: TargetProvider,
    //     Self: TargetProvider,
    //     (NT, Self::Target): NextTarget,
    //     Self: GenericAttach<'s, Shader = NT>,
    // {
    //     type Output = Builder<'s, <(NT, Self::Target) as NextTarget>::Next, CI, NO>;

    //     fn attach(mut self, shader: &'s Main<NT, CO, NO>) -> Self::Output {
    //         <Self as GenericAttach>::generic_attach(&mut self, &shader.0);
    //         Builder {
    //             _target_phantom: std::marker::PhantomData,
    //             _input_phantom: std::marker::PhantomData,
    //             _output_phantom: std::marker::PhantomData,
    //             vertex: self.vertex,
    //             tesselation_control: self.tesselation_control,
    //             tesselation_evaluation: self.tesselation_evaluation,
    //             geometry: self.geometry,
    //             fragment: self.fragment,
    //             compute: self.compute,
    //         }
    //     }
    // }
}

//endregion

// impl<'s, OldI: Type, OldO: Type, NewI: Type, NewO: Type> Attach<'s, Main<Vertex, NewI, NewO>> for Builder<'s, Vertex, OldI, OldO> {
//     type Updated = Self;

//     fn attach(self, shader: &'s Main<Vertex, NewI, NewO>) -> Self {
//         todo!()
//     }
// }

// impl<'s, I: Type, VO: Type> Attach<'s, tesselation::Control> for Builder<'s, Vertex, I, VO> {
//     type Result = Builder<'s, tesselation::Control, VO, TCO>;

//     fn attach(self, shader: &'s TesselationControlShader) -> Self::Result {
//         self.tesselation_control.push(shader);
//         self
//     }
// }

// impl Attach<Geometry> for Vertex {
//     type Result = Geometry;

//     fn attach(self, shader: Geometry) -> Self::Result {
//         todo!()
//     }
// }

// impl Attach<Fragment> for Vertex {
//     type Result = Fragment;

//     fn attach(self, shader: Fragment) -> Self::Result {
//         todo!()
//     }
// }

// // Tesselation attachments
// impl Attach<tesselation::Control> for tesselation::Control {
//     type Result = Self;

//     fn attach(self, shader: tesselation::Control) -> Self::Result {
//         todo!()
//     }
// }

// impl Attach<tesselation::Evaluation> for tesselation::Control {
//     type Result = tesselation::Evaluation;

//     fn attach(self, shader: tesselation::Control) -> Self::Result {
//         todo!()
//     }
// }

// impl Attach<tesselation::Evaluation> for tesselation::Evaluation {
//     type Result = Self;

//     fn attach(self, shader: tesselation::Evaluation) -> Self::Result {
//         todo!()
//     }
// }

// impl Attach<Geometry> for tesselation::Evaluation {
//     type Result = Geometry;

//     fn attach(self, shader: Geometry) -> Self::Result {
//         todo!()
//     }
// }

// impl Attach<Fragment> for tesselation::Evaluation {
//     type Result = Fragment;

//     fn attach(self, shader: Fragment) -> Self::Result {
//         todo!()
//     }
// }

// // Geometry attachment
// impl Attach<Geometry> for Geometry {
//     type Result = Self;

//     fn attach(self, shader: Geometry) -> Self::Result {
//         todo!()
//     }
// }

// impl Attach<Fragment> for Geometry {
//     type Result = Fragment;

//     fn attach(self, shader: Fragment) -> Self::Result {
//         todo!()
//     }
// }

// // Fragment attachment
// impl Attach<Fragment> for Fragment {
//     type Result = Self;

//     fn attach(self, shader: Fragment) -> Self::Result {
//         todo!()
//     }
// }
