use crate::glsl;
use crate::prelude::HList;
use crate::mode;
use crate::constraint;
use glsl::{layout, storage};
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
pub struct Variable<S, L, T, Store = mode::Phantom>(Store::Store<T>, PhantomData<(S, L)>)
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
    Store: mode::Storage;

impl<S, L, T> Default for Variable<S, L, T>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<S, L, T, Store> marker::Variable for Variable<S, L, T, Store>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
    Store: mode::Storage
{
    type Type = T;
}



#[derive(Clone, Copy, Debug)]
pub struct Binding<S, L, T, Store = mode::Phantom>(Store::Store<T>, PhantomData<(S, L)>)
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
    Store: mode::Storage;

impl<S, L, T> Default for Binding<S, L, T>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<S, L, T> Binding<S, L, T, mode::Inline>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

pub type UniformBinding<U, const LOCATION: usize, S = mode::Phantom> = Binding<storage::Uniform, layout::Location<LOCATION>, U, S>;
pub type UniformDefinition<U, const LOCATION: usize> = UniformBinding<U, LOCATION, mode::Inline>;


pub type InParameterBinding<T, const LOCATION: usize, S = mode::Phantom> = Binding<storage::In, layout::Location<LOCATION>, T, S>;
pub type OutParameterBinding<T, const LOCATION: usize, S = mode::Phantom> = Binding<storage::Out, layout::Location<LOCATION>, T, S>;

impl<T, const LOCATION: usize> OutParameterBinding<T, LOCATION>
where
    T: glsl::Type,
{
    fn matching_input(&self) -> InParameterBinding<T, LOCATION> {
        InParameterBinding::default().validate()
    }
}

impl<S, L, T> constraint::ConstFnValid for ((), Binding<S, L, T>)
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    const VALID: () = ();
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

impl<H, S, PT, CT, const PL: usize, const CL: usize, Store> constraint::ConstFnValid for (
        (H, Binding<S, layout::Location<PL>, PT, Store>), Binding<S, layout::Location<CL>, CT, Store>,
    )
where
    (H, Binding<S, layout::Location<PL>, PT, Store>): constraint::ConstFnValid,
    H: HList,
    S: Qualifier<Storage>,
    PT: glsl::Type,
    CT: glsl::Type,
    Store: mode::Storage,
{
    const VALID: () = are_locations_valid::<PT, PL, CT, CL>();
}

pub trait MatchingInputs {
    type Inputs: glsl::Parameters<storage::In>;

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
macro_rules! layout_qualifier {
    (location = $value:literal) => {
        crate::glsl::binding::marker::layout::Location<$value>
    };
}

#[macro_export]
macro_rules! storage_qualifier {
    (in) => {
        crate::glsl::binding::marker::storage::In
    };
    (out) => {
        crate::glsl::binding::marker::storage::Out
    };
    (uniform) => {
        crate::glsl::binding::marker::storage::Uniform
    };
}

#[macro_export]
macro_rules! type_qualifier {
    (vec2  ) => { crate::glsl::Vec2   };
    (vec3  ) => { crate::glsl::Vec3   };
    (vec4  ) => { crate::glsl::Vec4   };
    (ivec2 ) => { crate::glsl::IVec2  };
    (ivec3 ) => { crate::glsl::IVec3  };
    (ivec4 ) => { crate::glsl::IVec4  };
    (uvec2 ) => { crate::glsl::UVec2  };
    (uvec3 ) => { crate::glsl::UVec3  };
    (uvec4 ) => { crate::glsl::UVec4  };
    (dvec2 ) => { crate::glsl::DVec2  };
    (dvec3 ) => { crate::glsl::DVec3  };
    (dvec4 ) => { crate::glsl::DVec4  };
    (mat2  ) => { crate::glsl::Mat2   };
    (mat2x2) => { crate::glsl::Mat2x2 };
    (mat2x3) => { crate::glsl::Mat2x3 };
    (mat2x4) => { crate::glsl::Mat2x4 };
    (mat3  ) => { crate::glsl::Mat3   };
    (mat3x2) => { crate::glsl::Mat3x2 };
    (mat3x3) => { crate::glsl::Mat3x3 };
    (mat3x4) => { crate::glsl::Mat3x4 };
    (mat4  ) => { crate::glsl::Mat4   };
    (mat4x2) => { crate::glsl::Mat4x2 };
    (mat4x3) => { crate::glsl::Mat4x3 };
    (mat4x4) => { crate::glsl::Mat4x4 };
    (vec2   $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(vec2   $([$sizes])*), $size>  };
    (vec3   $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(vec3   $([$sizes])*), $size>  };
    (vec4   $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(vec4   $([$sizes])*), $size>  };
    (ivec2  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(ivec2  $([$sizes])*), $size>  };
    (ivec3  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(ivec3  $([$sizes])*), $size>  };
    (ivec4  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(ivec4  $([$sizes])*), $size>  };
    (uvec2  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(uvec2  $([$sizes])*), $size>  };
    (uvec3  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(uvec3  $([$sizes])*), $size>  };
    (uvec4  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(uvec4  $([$sizes])*), $size>  };
    (dvec2  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(dvec2  $([$sizes])*), $size>  };
    (dvec3  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(dvec3  $([$sizes])*), $size>  };
    (dvec4  $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(dvec4  $([$sizes])*), $size>  };
    (mat2   $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat2   $([$sizes])*), $size>  };
    (mat2x2 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat2x2 $([$sizes])*), $size>  };
    (mat2x3 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat2x3 $([$sizes])*), $size>  };
    (mat2x4 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat2x4 $([$sizes])*), $size>  };
    (mat3   $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat3   $([$sizes])*), $size>  };
    (mat3x2 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat3x2 $([$sizes])*), $size>  };
    (mat3x3 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat3x3 $([$sizes])*), $size>  };
    (mat3x4 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat3x4 $([$sizes])*), $size>  };
    (mat4   $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat4   $([$sizes])*), $size>  };
    (mat4x2 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat4x2 $([$sizes])*), $size>  };
    (mat4x3 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat4x3 $([$sizes])*), $size>  };
    (mat4x4 $([$sizes:literal])* [$size:literal]) => { crate::glsl::Array<type_qualifier!(mat4x4 $([$sizes])*), $size>  };
}

#[macro_export]
macro_rules! Bindings {
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*;) => {
        ((), crate::glsl::binding::Binding::<
            crate::storage_qualifier!($storage),
            crate::layout_qualifier!($qualifier = $value),
            crate::type_qualifier!($type $([$size])*)
        >
    )
    };
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);* ;) => {
        crate::Bindings! {
            @ ((), crate::glsl::binding::Binding::<
                crate::storage_qualifier!($storage),
                crate::layout_qualifier!($qualifier = $value),
                crate::type_qualifier!($type $([$size])*)
            >)
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);* 
        }
    };
    (@ $acc:ty => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*) => {
        ($acc, crate::glsl::binding::Binding::<
            crate::storage_qualifier!($storage),
            crate::layout_qualifier!($qualifier = $value),
            crate::type_qualifier!($type $([$size])*)
        >)
    };
    (@ $acc: ty => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);*) => {
        crate::Bindings! {
            @ ($acc, crate::glsl::binding::Binding::<
                crate::storage_qualifier!($storage),
                crate::layout_qualifier!($qualifier = $value),
                crate::type_qualifier!($type $([$size])*)
            >)
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);*
        }
    };
}

#[macro_export]
macro_rules! bindings {
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*;) => {
        crate::constraint::Valid::validated(
            ((), crate::glsl::binding::Binding::<
                crate::storage_qualifier!($storage),
                crate::layout_qualifier!($qualifier = $value),
                crate::type_qualifier!($type $([$size])*)
            >::default())
        )
    };
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);* ;) => {
        crate::bindings! {
            @
            crate::constraint::Valid::validated(
                ((), crate::glsl::binding::Binding::<
                    crate::storage_qualifier!($storage),
                    crate::layout_qualifier!($qualifier = $value),
                    crate::type_qualifier!($type $([$size])*)
                >::default())
            )
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);*
        }
    };
    (@ $acc:expr => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*) => {
        crate::constraint::Valid::validated(
            ($acc, crate::glsl::binding::Binding::<
                crate::storage_qualifier!($storage),
                crate::layout_qualifier!($qualifier = $value),
                crate::type_qualifier!($type $([$size])*)
            >::default())
        )
    };
    (@ $acc:expr => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);*) => {
        crate::bindings! {
            @
            crate::constraint::Valid::validated(
                ($acc, crate::glsl::binding::Binding::<
                    storage_qualifier!($storage),
                    layout_qualifier!($qualifier = $value),
                    type_qualifier!($type $([$size])*)
                >::default())
            )
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);*
        }
    };
}

#[macro_export]
macro_rules! inputs {
    (layout ($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        crate::bindings! {
            layout($qualifier = $value) in $type;
        }
    };
    (layout ($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        crate::bindings! {
            layout ($qualifier = $value) in $type;
            $(layout ($qualifiers = $values) in $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Inputs {
    (layout ($qualifier:ident = $value:literal) $type:ident;) => {
        crate::Bindings! {
            layout($qualifier = $value) in $type;
        }
    };
    (layout ($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        crate::Bindings! {
            layout($qualifier = $value) in $type;
            $(layout ($qualifiers = $values) in $types);* ;
        }
    };
}

#[macro_export]
macro_rules! outputs {
    (layout ($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        crate::bindings! {
            layout($qualifier = $value) out $type;
        }
    };
    (layout ($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        crate::bindings! {
            layout ($qualifier = $value) out $type;
            $(layout($qualifiers = $values) out $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Outputs {
    (layout($qualifier:ident = $value:literal) $type:ident ;) => {
        crate::Bindings! {
            layout($qualifier = $value) out $type;
        }
    };
    (layout($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        crate::Bindings! {
            layout ($qualifier = $value) out $type;
            $(layout($qualifiers = $values) out $types);* ;
        }
    };
}

#[macro_export]
macro_rules! uniforms {
    (layout($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        crate::bindings! {
            layout($qualifier = $value) uniform $type;
        }
    };
    (layout($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        crate::bindings! {
            layout($qualifier = $value) uniform $type;
            $(layout($qualifiers = $values) uniform $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Uniforms {
    (layout($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        crate::Bindings! {
            layout($qualifier = $value) uniform $type;
        }
    };
    (layout($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        crate::Bindings! {
            layout($qualifier = $value) uniform $type;
            $(layout($qualifiers = $values) uniform $types);* ;
        }
    };
}

#[macro_export]
/// Unpack bindings into separate variables
macro_rules! unpack {
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