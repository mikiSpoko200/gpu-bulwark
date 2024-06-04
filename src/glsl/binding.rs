use crate::glsl;
use crate::prelude::HList;
use std::marker::PhantomData;

pub mod marker {
    use crate::glsl;

    pub trait Qualifier<Type> {}

    /// glsl4.60 spec: 4.3. Storage Qualifiers
    #[derive(Debug)]
    pub struct Storage;

    /// glsl4.60 spec: 4.4. Layout Qualifiers
    #[derive(Debug)]
    pub struct Layout;

    /// glsl4.60 spec: 4.5. Interpolation Qualifiers
    #[derive(Debug)]
    pub struct Interpolation;

    pub trait ValidationStatus {}

    /// GL / GLSL binding target --
    pub trait Variable {
        type Type: glsl::Type;
    }

    /// storage qualifiers
    pub mod storage {
        use super::{Qualifier, Storage};

        #[derive(Debug)]
        /// linkage into a shader from a previous stage, variable is copied in.
        pub struct In;

        #[derive(Debug)]
        /// linkage out of a shader to a subsequent stage, variable is copied out
        pub struct Out;

        #[derive(Debug)]
        /// Value does not change across the primitive being processed, uniforms form the linkage between a shader, API, and the application
        pub struct Uniform;

        #[derive(Debug)]
        /// value is stored in a buffer object, and can be read or written both by shader invocations and the API
        pub struct Buffer;

        impl Qualifier<Storage> for In {}
        impl Qualifier<Storage> for Out {}
        impl Qualifier<Storage> for Uniform {}
        impl Qualifier<Storage> for Buffer {}
    }

    pub mod layout {
        use super::{Layout, Qualifier};

        pub struct Location<const N: usize>;
        impl<const N: usize> Qualifier<Layout> for Location<N> {}

        pub struct Binding<const N: usize>;
        impl<const N: usize> Qualifier<Layout> for Binding<N> {}
    }
}

pub use marker::{Layout, Qualifier, Storage};

#[derive(Clone, Copy, Debug)]
pub struct Unvalidated;
#[derive(Clone, Copy, Debug)]
pub struct Validated;

impl marker::ValidationStatus for Unvalidated {}
impl marker::ValidationStatus for Validated {}

#[derive(Clone, Copy, Debug)]
pub struct Phantom;

#[derive(Clone, Copy, Debug)]
pub struct Inline;

impl crate::prelude::Storage for Phantom {
    type Store<T> = PhantomData<T>;
}

impl crate::prelude::Storage for Inline {
    type Store<T> = T;
}

#[derive(Clone, Copy, Debug)]
pub struct Variable<S, L, T, Store = Phantom>(Store::Store<T>, PhantomData<(Storage, Layout)>)
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
    Store: crate::prelude::Storage;

impl<S, L, T, Store> Variable<S, L, T, Store>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
    Store: crate::prelude::Storage,
{
    fn new() -> Self {
        Self(Default::default(), PhantomData)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Binding<V, Valid = Validated>(V, PhantomData<Valid>)
where
    V: marker::Variable,
    Valid: marker::ValidationStatus;

impl<Target> Default for Binding<Target, Unvalidated>
where
    Target: marker::Variable,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Target> Binding<Target, Unvalidated>
where
    Target: marker::Variable,
{
    pub fn new() -> Self {
        Self(Default::default(), PhantomData)
    }

    fn validate(self) -> Binding<Target> {
        Binding(self.0, PhantomData)
    }
}

use marker::{layout, storage};

pub type UniformBinding<U, const LOCATION: usize, V = Validated, S = Phantom> =
    Binding<Variable<storage::Uniform, layout::Location<LOCATION>, U, S>, V>;
pub type InParameterBinding<T, const LOCATION: usize, V = Validated, S = Phantom> =
    Binding<Variable<storage::In, layout::Location<LOCATION>, T, S>, V>;
pub type OutParameterBinding<T, const LOCATION: usize, V = Validated, S = Phantom> =
    Binding<Variable<storage::Out, layout::Location<LOCATION>, T, S>, V>;

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

impl<Target, const LOCATION: usize> Bindings<Unvalidated> for ((), Binding<Target, Unvalidated>)
where
    Target: marker::Variable,
{
    const LOCATIONS_VALID: () = ();
    type Validated = ((), Binding<Target, Validated>);

    fn validate(self) -> Self::Validated {
        let _: () = Self::LOCATIONS_VALID;
        ((), self.1.validate())
    }
}

impl<Target> Bindings<Validated> for ((), Binding<Target, Validated>)
where
    Target: marker::Variable,
{
    const LOCATIONS_VALID: () = ();
    type Validated = Self;

    fn validate(self) -> Self::Validated {
        self
    }
}

const fn are_locations_valid<PT, const PREV_LOCATION: usize, CT, const CURR_LOCATION: usize>()
where
    PT: glsl::Type,
    CT: glsl::Type,
{
    assert!(
        !(PREV_LOCATION > CURR_LOCATION + CT::LOCATION_COUNT),
        "locations must be specified in strictly increasing order"
    );
    assert!(
        PREV_LOCATION + PT::LOCATION_COUNT <= CURR_LOCATION,
        "locations overlap"
    );
}

impl<H, S, PT, CT, const PREV_LOCATION: usize, const CURR_LOCATION: usize, Store> Bindings<Unvalidated> 
    for (
        (H, Binding<Variable<S, layout::Location<PREV_LOCATION>, PT, Store>, Validated>),
        Binding<Variable<S, layout::Location<CURR_LOCATION>, CT, Store>, Validated>,
    )
where
    (H, Binding<Variable<S, layout::Location<PREV_LOCATION>, PT, Store>, Validated>): Bindings<Validated>,
    H: HList,
    S: Qualifier<Storage>,
    Store: crate::prelude::Storage,
    PT: marker::Variable,
    CT: marker::Variable,
{
    const LOCATIONS_VALID: () =
        are_locations_valid::<PT::Type, PREV_LOCATION, CT::Type, CURR_LOCATION>();
    type Validated = (
        <(H, Binding<Variable<S, layout::Location<PREV_LOCATION>, PT, Store>, Validated>) as Bindings<Validated>>::Validated,
        Binding<Variable<S, layout::Location<CURR_LOCATION>, CT, Store>, Validated>,
    );

    fn validate(self) -> Self::Validated {
        let _: () = Self::LOCATIONS_VALID;
        let (head, binding) = self;
        (head.validate(), binding.validate())
    }
}

impl<H, S, PT, CT, const PREV_LOCATION: usize, const CURR_LOCATION: usize, Store> Bindings<Validated>
    for ((
        H, Binding<Variable<S, layout::Location<PREV_LOCATION>, PT, Store>, Validated>),
        Binding<Variable<S, layout::Location<CURR_LOCATION>, CT, Store>, Validated>,
    )
where
    (H, Binding<Variable<S, layout::Location<PREV_LOCATION>, PT, Store>, Validated>): Bindings<Validated>,
    H: HList,
    S: Qualifier<Storage>,
    Store: crate::prelude::Storage,
    PT: marker::Variable,
    CT: marker::Variable,
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
        crate::glsl::binding::Binding<crate::glsl::binding::In, $type>
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
