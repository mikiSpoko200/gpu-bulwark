use std::marker::PhantomData;
use crate::{glsl, object::shader::parameters, prelude::HList};

mod marker {
    use crate::glsl;

    pub trait ValidationStatus { }
    pub trait ParameterQualifier { }
    pub trait Target {
        type Type: glsl::Type;
    }

}

pub use marker::ParameterQualifier;

#[derive(Clone, Copy, Debug)]
pub struct Unvalidated;
#[derive(Clone, Copy, Debug)]
pub struct Validated;

impl marker::ValidationStatus for Unvalidated {} 
impl marker::ValidationStatus for Validated {} 

#[derive(Clone, Copy, Debug)]
pub struct In;

#[derive(Clone, Copy, Debug)]
pub struct Out;

impl marker::ParameterQualifier for In { }
impl marker::ParameterQualifier for Out { }


#[derive(Clone, Copy, Debug)]
pub struct Parameter<Q, T>(PhantomData<(Q, T)>)
where
    T: glsl::Type,
    Q: marker::ParameterQualifier,
;

impl<T, Q> marker::Target for Parameter<Q, T>
where 
    T: glsl::Type, 
    Q: marker::ParameterQualifier
{
    type Type = T;
}

#[derive(Clone, Copy, Debug)]
pub struct Uniform<T>(PhantomData<T>) where T: glsl::Uniform;

impl<T> marker::Target for Uniform<T> where T: glsl::Uniform {
    type Type = T;
}

#[derive(Clone, Copy, Debug)]
pub struct Binding<Target, const LOCATION: usize, Valid=Validated>(PhantomData<(Target, Valid)>)
where 
    Target: marker::Target,
    Valid: marker::ValidationStatus,
;


impl<Target, const LOCATION: usize> Default for Binding<Target, LOCATION, Unvalidated>
where 
    Target: marker::Target,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Target, const LOCATION: usize> Binding<Target, LOCATION, Unvalidated>
where 
    Target: marker::Target,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }

    fn validate(self) -> Binding<Target, LOCATION> {
        Binding(PhantomData)
    }
}

pub type UniformBinding<U, const LOCATION: usize, V=Validated> = Binding<Uniform<U>, LOCATION, V>;
pub type InParameterBinding<T, const LOCATION: usize, V=Validated> = Binding<Parameter<In, T>, LOCATION, V>;
pub type OutParameterBinding<T, const LOCATION: usize, V=Validated> = Binding<Parameter<Out, T>, LOCATION, V>;

impl<T, const LOCATION: usize> OutParameterBinding<T, LOCATION>
where
    T: glsl::Type,
{
    fn matching_input(&self) -> InParameterBinding<T, LOCATION> {
        InParameterBinding::default().validate()
    }
}

pub trait Bindings<ValidationStatus>: HList {
    const LOCATIONS_VALID: ();
    type Validated: HList;
    
    fn validate(self) -> Self::Validated;
}

impl<Target, const LOCATION: usize> Bindings<Unvalidated> for ((), Binding<Target, LOCATION, Unvalidated>)
where
    Target: marker::Target,
{
    const LOCATIONS_VALID: () = ();
    type Validated = ((), Binding<Target, LOCATION, Validated>);
    
    fn validate(self) -> Self::Validated {
        let _: () = Self::LOCATIONS_VALID;
        ((), self.1.validate())
    }
}

impl<Target, const LOCATION: usize> Bindings<Validated> for ((), Binding<Target, LOCATION, Validated>)
where
    Target: marker::Target,
{
    const LOCATIONS_VALID: () = ();
    type Validated = Self;
    
    fn validate(self) -> Self::Validated {
        self
    }
}

const fn are_locations_valid<PT, const PREV_LOCATION: usize, CT, const CURR_LOCATION: usize>() -> ()
where
    PT: glsl::Type,
    CT: glsl::Type,
{
    assert!(!(PREV_LOCATION > CURR_LOCATION + CT::LOCATION_COUNT), "locations must be specified in strictly increasing order");
    assert!(PREV_LOCATION + PT::LOCATION_COUNT <= CURR_LOCATION, "locations overlap");
    ()
}

impl<H, PT, CT, const PREV_LOCATION: usize, const CURR_LOCATION: usize> Bindings<Unvalidated> for ((H, Binding<PT, PREV_LOCATION, Validated>), Binding<CT, CURR_LOCATION, Unvalidated>)
where
    (H, Binding<PT, PREV_LOCATION, Validated>): Bindings<Validated>,
    H: HList,
    PT: marker::Target,
    CT: marker::Target,
{
    const LOCATIONS_VALID: () = are_locations_valid::<PT::Type, PREV_LOCATION, CT::Type, CURR_LOCATION>();
    type Validated = (<(H, Binding<PT, PREV_LOCATION, Validated>) as Bindings<Validated>>::Validated, Binding<CT, CURR_LOCATION, Validated>);
    
    fn validate(self) -> Self::Validated {
        let _: () = Self::LOCATIONS_VALID;
        let (head, binding) = self;
        (head.validate(), binding.validate())
    }
}

impl<H, PT, CT, const PREV_LOCATION: usize, const CURR_LOCATION: usize> Bindings<Validated> for ((H, Binding<PT, PREV_LOCATION, Validated>), Binding<CT, CURR_LOCATION, Validated>)
where
    (H, Binding<PT, PREV_LOCATION, Validated>): Bindings<Validated>,
    H: HList,
    PT: marker::Target,
    CT: marker::Target,
{
    const LOCATIONS_VALID: () = ();
    type Validated = Self;
    
    fn validate(self) -> Self::Validated {
        self
    }
}

pub trait MatchingInputs {
    type Inputs: parameters::Parameters<In>;

    fn matching_intputs(&self) -> Self::Inputs;
}

impl MatchingInputs for () {
    type Inputs = ();

    fn matching_intputs(&self) -> Self::Inputs {
        ()
    }
}

impl<H, T, const LOCATION: usize> MatchingInputs for (H, OutParameterBinding<T, LOCATION>)
where
    H: MatchingInputs,
    T: glsl::Type,
{
    type Inputs = (H::Inputs, InParameterBinding<T, LOCATION>);

    fn matching_intputs(&self) -> Self::Inputs {
        let (head, tail) = self;
        (head.matching_intputs(), tail.matching_input())
    }
}

#[macro_export]
macro_rules! binding_type {
    (in, $type: ty) => {
        crate::glsl::binding::Parameter<crate::glsl::binding::In, $type>
    };
    (out, $type: ty) => {
        crate::glsl::binding::Parameter<crate::glsl::binding::Out, $type>
    };
    (uniform, $type: ty) => {
        crate::glsl::binding::Uniform<$type>
    };
}

#[macro_export]
macro_rules! inputs {
    (layout (location=$location: expr) $type: ty $(;)?) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(((), crate::glsl::binding::Binding::<binding_type!(in, $type), $location, _>::new()))
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        inputs!(@ crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(((), crate::glsl::binding::Binding::<binding_type!(in, $type), $location, _>::default())), $(layout (location=$locations) $types);*)
    };
    (@ $acc: expr, layout (location=$location: expr) $type: ty) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(($acc, crate::glsl::binding::Binding::<binding_type!(in, $type), $location, _>::default()))
    };
    (@ $acc: expr, layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);*) => {
        inputs!(@ crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(($acc, crate::glsl::binding::Binding::<binding_type!(in, $type), $location, _>::default())), $(layout (location=$locations) $types);*)
    };
}

#[macro_export]
macro_rules! outputs {
    (layout (location=$location: expr) $type: ty $(;)?) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(((), crate::glsl::binding::Binding::<binding_type!(out, $type), $location, _>::new()))
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        inputs!(@ crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(((), crate::glsl::binding::Binding::<binding_type!(out, $type), $location, _>::default())), $(layout (location=$locations) $types);*)
    };
    (@ $acc: expr, layout (location=$location: expr) $type: ty) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(($acc, crate::glsl::binding::Binding::<binding_type!(out, $type), $location, _>::default()))
    };
    (@ $acc: expr, layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);*) => {
        inputs!(@ crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(($acc, crate::glsl::binding::Binding::<binding_type!(out, $type), $location, _>::default())), $(layout (location=$locations) $types);*)
    };
}

#[macro_export]
macro_rules! uniforms {
    (layout (location=$location: expr) $type: ty $(;)?) => {
        ((), crate::glsl::binding::Binding::<binding_type!(uniform, $type), $location, _>::new()).validate()
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        uniforms!(@ crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(((), crate::glsl::binding::Binding::<binding_type!(uniform, $type), $location, _>::default())), $(layout (location=$locations) $types);*)
    };
    (@ $acc: expr, layout (location=$location: expr) $type: ty) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(($acc, crate::glsl::binding::Binding::<binding_type!(uniform, $type), $location, _>::default()))
    };
    (@ $acc: expr, layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);*) => {
        uniforms!(@ crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(($acc, crate::glsl::binding::Binding::<binding_type!(uniform, $type), $location, _>::default())), $(layout (location=$locations) $types);*)
    };
}

#[macro_export]
macro_rules! locations {
    ($ident: ident $(,)?) => {
        $ident
    };
    ($ident: ident, $($idents: tt),* $(,)?) => {
        locations!(@ ((), $ident), $($idents),*)
    };
    (@ $acc: tt, $ident: tt $(,)?) => {
        ($acc, $ident)
    };
    (@ $acc: tt, $ident: ident, $($idents: tt),* $(,)?) => {
        locations!(@ ($acc, $ident), $($idents),*)
    };
}

#[macro_export]
macro_rules! glsl {
    (layout (location=$location: expr) $target: ident $ident: ident: $type: ty;) => {
        let $ident = crate::glsl::binding::Binding::<binding_type!($target, $type), $location, _>::new();
    };
    (layout (location=$location: expr) $target: ident $ident: ident: $type: ty; $(layout(location=$locations: expr) $targets: ident $idents: ident: $types: ty);* ;) => {
        let $ident = crate::glsl::binding::Binding::<binding_type!($target, $type), $location, _>::new();
        glsl! {
            $(layout(location=$locations) $targets $idents: $types);* ;
        }
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_multiple_one_sized_locations() {
        let _ = inputs! {
            layout(location=0) glsl::Vec3;
            layout(location=0) glsl::Vec3;
        };
        // this test should not compile.
        assert!(false);
    }
}