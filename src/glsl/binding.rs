use std::marker::PhantomData;
use crate::glsl; 
use crate::prelude::HList;

mod marker {
    use crate::{glsl, types::Primitive};

    pub trait ValidationStatus { }
    pub trait ParameterQualifier { }

    /// GL / GLSL binding target -- 
    pub trait Binding
    where
        Self::GlType: glsl::compatible::Compatible<Self::GlslType>,
    
    {
        type GlslType: glsl::Type;
        type GlType;
        type Storage: Storage;
    }

    pub trait Storage {
        type Store<T>;
    }
}

pub use marker::ParameterQualifier;
pub use marker::Storage;

#[derive(Clone, Copy, Debug)]
pub struct Unvalidated;
#[derive(Clone, Copy, Debug)]
pub struct Validated;

impl marker::ValidationStatus for Unvalidated { } 
impl marker::ValidationStatus for Validated { } 

#[derive(Clone, Copy, Debug)]
pub struct In;

#[derive(Clone, Copy, Debug)]
pub struct Out;

impl marker::ParameterQualifier for In { }
impl marker::ParameterQualifier for Out { }

#[derive(Clone, Copy, Debug)]
pub struct Phantom;

#[derive(Clone, Copy, Debug)]
pub struct Inline;

impl marker::Storage for Phantom {
    type Store<T> = PhantomData<T>;
}

impl marker::Storage for Inline {
    type Store<T> = T;
}

#[derive(Clone, Copy, Debug)]
pub struct Parameter<Q, T, S=Phantom>(S::Store<T>, PhantomData<Q>)
where
    T: glsl::Type,
    Q: marker::ParameterQualifier,
    S: marker::Storage,
;

impl<T, Q, S> marker::Binding for Parameter<Q, T, S>
where 
    T: glsl::Type, 
    Q: marker::ParameterQualifier,
    S: marker::Storage,
{
    type GlslType = T;
}

impl<T, Q> Default for Parameter<Q, T>
where 
    T: glsl::Type, 
    Q: marker::ParameterQualifier,
{
    fn default() -> Self {
        Self(Default::default(), PhantomData)
    }
}


#[derive(Clone, Copy, Debug)]
pub struct Uniform<T, S=Phantom>(S::Store<T>)
where
    T: glsl::Uniform,
    S: marker::Storage,
;

impl<T, S> marker::Binding for Uniform<T, S>
where
    T: glsl::Uniform,
    S: marker::Storage,
{
    type GlslType = T;
}

#[derive(Clone, Copy, Debug)]
pub struct Binding<Target, const LOCATION: usize, Valid=Validated>(Target, PhantomData<Valid>)
where 
    Target: marker::Binding,
    Valid: marker::ValidationStatus,
;


impl<Target, const LOCATION: usize> Default for Binding<Target, LOCATION, Unvalidated>
where 
    Target: marker::Binding,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Target, const LOCATION: usize> Binding<Target, LOCATION, Unvalidated>
where 
    Target: marker::Binding,
{
    pub fn new() -> Self {
        Self(Default::default(), PhantomData)
    }
    
    fn validate(self) -> Binding<Target, LOCATION> {
        Binding(self.0, PhantomData)
    }
}


pub type UniformBinding<U, const LOCATION: usize, V=Validated, S=Phantom> = Binding<Uniform<U, S>, LOCATION, V>;
pub type InParameterBinding<T, const LOCATION: usize, V=Validated, S=Phantom> = Binding<Parameter<In, T, S>, LOCATION, V>;
pub type OutParameterBinding<T, const LOCATION: usize, V=Validated, S=Phantom> = Binding<Parameter<Out, T, S>, LOCATION, V>;

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
    Target: marker::Binding,
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
    Target: marker::Binding,
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
    PT: marker::Binding,
    CT: marker::Binding,
{
    const LOCATIONS_VALID: () = are_locations_valid::<PT::GlslType, PREV_LOCATION, CT::GlslType, CURR_LOCATION>();
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
    PT: marker::Binding,
    CT: marker::Binding,
{
    const LOCATIONS_VALID: () = ();
    type Validated = Self;
    
    fn validate(self) -> Self::Validated {
        self
    }
}

pub trait MatchingInputs {
    type Inputs: glsl::Parameters<In>;

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
macro_rules! Bindings {
    ([$kind: ident] layout (location=$location: expr) $type: ty ;) => {
        ((), crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location>)
    };
    ([$kind: ident] layout (location=$location: expr) $type: ty; $([$kinds: ident] layout (location=$locations: expr) $types: ty);* ;) => {
        crate::Bindings! {
            @ ((), crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location>)
            =>
            $([$kinds] layout (location=$locations) $types);*
        }
    };
    (@ $acc: ty => [$kind: ident] layout (location=$location: expr) $type: ty) => {
        ($acc, crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location>)
    };
    (@ $acc: ty => [$kind: ident] layout (location=$location: expr) $type: ty; $([$kinds: ident] layout (location=$locations: expr) $types: ty);*) => {
        crate::Bindings! {
            @ ($acc, crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location>)
            =>
            $([$kinds] layout (location=$locations) $types);*
        }
    };
}

#[macro_export]
macro_rules! bindings {
    ([$kind: ident] layout (location=$location: expr) $type: ty ;) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(
            ((), crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location, _>::new())
        )
    };
    ([$kind: ident] layout (location=$location: expr) $type: ty; $([$kinds: ident] layout (location=$locations: expr) $types: ty);* ;) => {
        crate::bindings! {
            @ 
            crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(
                ((), crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location, _>::default())
            )
            =>
            $([$kinds] layout (location=$locations) $types);* 
        }
    };
    (@ $acc: expr => [$kind: ident] layout (location=$location: expr) $type: ty) => {
        crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(
            ($acc, crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location, _>::default())
        )
    };
    (@ $acc: expr => [$kind: ident] layout (location=$location: expr) $type: ty; $([$kinds: ident] layout (location=$locations: expr) $types: ty);*) => {
        crate::bindings! {
            @
            crate::glsl::binding::Bindings::<crate::glsl::binding::Unvalidated>::validate(
                ($acc, crate::glsl::binding::Binding::<crate::binding_type!($kind, $type), $location, _>::default())
            )
            =>
            $([$kinds] layout (location=$locations) $types);*
        }
    };
}


#[macro_export]
macro_rules! inputs {
    (layout (location=$location: expr) $type: ty $(;)?) => {
        crate::bindings! {
            [in] layout(location=$location) $type;
        }
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::bindings! { 
            [in] layout (location=$location) $type;
            $([in] layout (location=$locations) $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Inputs {
    (layout (location=$location: expr) $type: ty ;) => {
        crate::Bindings!{
            [in] layout (location=$location) $type;
        }
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::Bindings!{
            [in] layout (location=$location) $type;
            $([in] layout (location=$locations) $types);* ;
        }
    };
}

#[macro_export]
macro_rules! outputs {
    (layout (location=$location: expr) $type: ty $(;)?) => {
        crate::bindings! {
            [out] layout(location=$location) $type;
        }
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::bindings! { 
            [out] layout (location=$location) $type;
            $([out] layout (location=$locations) $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Outputs {
    (layout (location=$location: expr) $type: ty ;) => {
        crate::Bindings!{
            [out] layout (location=$location) $type;
        }
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::Bindings!{
            [out] layout (location=$location) $type;
            $([out] layout (location=$locations) $types);* ;
        }
    };
}

#[macro_export]
macro_rules! uniforms {
    (layout (location=$location: expr) $type: ty $(;)?) => {
        crate::bindings! {
            [uniform] layout(location=$location) $type;
        }
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::bindings! { 
            [uniform] layout (location=$location) $type;
            $([uniform] layout (location=$locations) $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Uniforms {
    (layout (location=$location: expr) $type: ty ;) => {
        crate::Bindings!{
            [uniform] layout (location=$location) $type;
        }
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::Bindings!{
            [uniform] layout (location=$location) $type;
            $([uniform] layout (location=$locations) $types);* ;
        }
    };
    (layout (location=$location: expr) $gl: ty as $glsl: ty ;) => {
        ((), crate::glsl::uniform)
    };
    (layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);* ;) => {
        crate::Bindings! {
            @ ((), crate::glsl::binding::Binding::<crate::binding_type!(uniform, $type), $location>)
            =>
            $(layout (location=$locations) $types);*
        }
    };
    (@ $acc: ty => layout (location=$location: expr) $type: ty) => {
        ($acc, crate::glsl::binding::Binding::<crate::binding_type!(uniform, $type), $location>)
    };
    (@ $acc: ty => layout (location=$location: expr) $type: ty; $(layout (location=$locations: expr) $types: ty);*) => {
        crate::Bindings! {
            @ ($acc, crate::glsl::binding::Binding::<crate::binding_type!(uniform, $type), $location>)
            =>
            $([$kinds] layout (location=$locations) $types);*
        }
    };
}

#[macro_export]
macro_rules! locations {
    ($ident: ident $(,)?) => {
        ((), $ident)
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
    use super::glsl;

    #[test]
    fn check_multiple_one_sized_locations() {
        let _ = crate::inputs! {
            layout(location=0) glsl::Vec3;
            layout(location=0) glsl::Vec3;
        };
        // this test should not compile.
        assert!(false);
    }
}