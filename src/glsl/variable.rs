use crate::prelude::internal::*;

use crate::glsl;
use crate::hlist::lhlist as hlist;
use crate::md;
use crate::constraint;

use hlist::Base as HList;

#[hi::marker]
pub trait Qualifier<Type> { }

/// glsl4.60 spec: 4.3. Storage Qualifiers
#[derive(Debug)]
pub enum Storage { }

/// glsl4.60 spec: 4.4. Layout Qualifiers
#[derive(Debug)]
pub enum Layout { }

/// glsl4.60 spec: 4.5. Interpolation Qualifiers
#[derive(Debug)]
pub enum Interpolation { }

/// storage qualifiers
pub mod storage {
    use super::{Qualifier, Storage};

    #[derive(Debug)]
    /// linkage into a shader from a previous stage, variable is copied in.
        pub enum In { }

    #[derive(Debug)]
    /// linkage out of a shader to a subsequent stage, variable is copied out
    pub enum Out { }

    #[derive(Debug)]
    /// Value does not change across the primitive being processed, uniforms form the linkage between a shader, API, and the application
    pub enum Uniform { }

    #[derive(Debug)]
    /// value is stored in a buffer object, and can be read or written both by shader invocations and the API
    pub enum Buffer { }

    hi::denmark! { In as Qualifier<Storage> }
    hi::denmark! { Out as Qualifier<Storage> }
    hi::denmark! { Uniform as Qualifier<Storage> }
    hi::denmark! { Buffer as Qualifier<Storage> }
}

pub mod layout {
    use super::{Layout, Qualifier};

    pub enum Binding<const N: usize> { }
    impl<const N: usize> Qualifier<Layout> for Binding<N> { }

    pub enum Location<const N: usize> { }
    impl<const N: usize> Qualifier<Layout> for Location<N> { }

    // pub enum Variable<const N: usize> { }
    // impl<const N: usize> Qualifier<Layout> for Variable<N> {}
}

#[derive(Clone, Copy, Debug)]
pub struct Variable<S, L, T, Store = md::Phantom>(Store::Store<T>, PhantomData<(S, L)>)
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
    Store: md::Storage;

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

impl<S, L, T> Variable<S, L, T, md::Phantom>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    pub const fn default() -> Self {
        Self(PhantomData, PhantomData)
    }
}

impl<S, L, T> Variable<S, L, T, md::Inline>
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    pub fn new(value: T) -> Self {
        Self(value, PhantomData)
    }
}

pub type TransparentUniformVariable<U, const LOCATION: usize, S = md::Phantom> = Variable<storage::Uniform, layout::Location<LOCATION>, U, S>;
pub type OpaqueUniformVariable<U, const BINDING: usize> = Variable<storage::Uniform, layout::Binding<BINDING>, U, md::Phantom>;

pub type UniformDefinition<U, const LOCATION: usize> = TransparentUniformVariable<U, LOCATION, md::Inline>;


pub type InVariable<T, const LOCATION: usize, S = md::Phantom> = Variable<storage::In, layout::Location<LOCATION>, T, S>;
pub type OutVariable<T, const LOCATION: usize, S = md::Phantom> = Variable<storage::Out, layout::Location<LOCATION>, T, S>;

pub type SamplerVariable<Target, Output, const BINDING: usize> = OpaqueUniformVariable<glsl::GSampler<Target, Output>, BINDING>;

impl<T, const LOCATION: usize> OutVariable<T, LOCATION>
where
    T: glsl::bounds::TransparentType,
{
    fn matching_input(&self) -> InVariable<T, LOCATION> {
        InVariable::default()
    }
}

impl<S, L, T> constraint::ConstFnValid for ((), Variable<S, L, T>)
where
    T: glsl::Type,
    S: Qualifier<Storage>,
    L: Qualifier<Layout>,
{
    const VALID: () = ();
}

const fn are_locations_valid<PT, const PREV_LOCATION: usize, CT, const CURR_LOCATION: usize>()
where
    PT: glsl::bounds::TransparentType,
    CT: glsl::bounds::TransparentType,
{
    assert!(
        !(PREV_LOCATION > CURR_LOCATION + CT::N_USED_LOCATIONS),
        "locations must be specified in strictly increasing order"
    );
    assert!(
        PREV_LOCATION + PT::N_USED_LOCATIONS <= CURR_LOCATION,
        "locations overlap"
    );
}

impl<H, S, PT, CT, const PL: usize, const CL: usize, Store> constraint::ConstFnValid for (
    (H, Variable<S, layout::Location<PL>, PT, Store>), Variable<S, layout::Location<CL>, CT, Store>,
)
where
    (H, Variable<S, layout::Location<PL>, PT, Store>): constraint::ConstFnValid,
    H: HList,
    S: Qualifier<Storage>,
    PT: glsl::bounds::TransparentType,
    CT: glsl::bounds::TransparentType,
    Store: md::Storage,
{
    const VALID: () = are_locations_valid::<PT, PL, CT, CL>();
}

pub trait MatchingInputs {
    type Inputs: glsl::Parameters<storage::In>;

    fn matching_inputs(&self) -> Self::Inputs;
}

impl MatchingInputs for () {
    type Inputs = ();

    fn matching_inputs(&self) -> Self::Inputs {
        ()
    }
}

impl<H, T, const LOCATION: usize> MatchingInputs for (H, OutVariable<T, LOCATION>)
where
    H: MatchingInputs,
    T: glsl::bounds::Parameter<storage::In>,
{
    type Inputs = (H::Inputs, InVariable<T, LOCATION>);

    fn matching_inputs(&self) -> Self::Inputs {
        let (head, tail) = self;
        (head.matching_inputs(), tail.matching_input())
    }
}

#[macro_export]
macro_rules! layout_qualifier {
    (location = $value:literal) => {
        $crate::glsl::variable::layout::Location<$value>
    };
    (binding = $value:literal) => {
        $crate::glsl::variable::layout::Binding<$value>
    };
}

#[macro_export]
macro_rules! storage_qualifier {
    (in) => {
        $crate::glsl::variable::storage::In
    };
    (out) => {
        $crate::glsl::variable::storage::Out
    };
    (uniform) => {
        $crate::glsl::variable::storage::Uniform
    };
}

#[macro_export]
macro_rules! type_qualifier {
    (float ) => { f32   };
    (double) => { f64   };
    (int   ) => { i32   };
    (uint  ) => { u32   };
    (vec2  ) => { $crate::glsl::Vec2   };
    (vec3  ) => { $crate::glsl::Vec3   };
    (vec4  ) => { $crate::glsl::Vec4   };
    (ivec2 ) => { $crate::glsl::IVec2  };
    (ivec3 ) => { $crate::glsl::IVec3  };
    (ivec4 ) => { $crate::glsl::IVec4  };
    (uvec2 ) => { $crate::glsl::UVec2  };
    (uvec3 ) => { $crate::glsl::UVec3  };
    (uvec4 ) => { $crate::glsl::UVec4  };
    (dvec2 ) => { $crate::glsl::DVec2  };
    (dvec3 ) => { $crate::glsl::DVec3  };
    (dvec4 ) => { $crate::glsl::DVec4  };
    (mat2  ) => { $crate::glsl::Mat2   };
    (mat2x2) => { $crate::glsl::Mat2x2 };
    (mat2x3) => { $crate::glsl::Mat2x3 };
    (mat2x4) => { $crate::glsl::Mat2x4 };
    (mat3  ) => { $crate::glsl::Mat3   };
    (mat3x2) => { $crate::glsl::Mat3x2 };
    (mat3x3) => { $crate::glsl::Mat3x3 };
    (mat3x4) => { $crate::glsl::Mat3x4 };
    (mat4  ) => { $crate::glsl::Mat4   };
    (mat4x2) => { $crate::glsl::Mat4x2 };
    (mat4x3) => { $crate::glsl::Mat4x3 };
    (mat4x4) => { $crate::glsl::Mat4x4 };
    (vec2   $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(vec2   $([$sizes])*), $size>  };
    (vec3   $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(vec3   $([$sizes])*), $size>  };
    (vec4   $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(vec4   $([$sizes])*), $size>  };
    (ivec2  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(ivec2  $([$sizes])*), $size>  };
    (ivec3  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(ivec3  $([$sizes])*), $size>  };
    (ivec4  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(ivec4  $([$sizes])*), $size>  };
    (uvec2  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(uvec2  $([$sizes])*), $size>  };
    (uvec3  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(uvec3  $([$sizes])*), $size>  };
    (uvec4  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(uvec4  $([$sizes])*), $size>  };
    (dvec2  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(dvec2  $([$sizes])*), $size>  };
    (dvec3  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(dvec3  $([$sizes])*), $size>  };
    (dvec4  $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(dvec4  $([$sizes])*), $size>  };
    (mat2   $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat2   $([$sizes])*), $size>  };
    (mat2x2 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat2x2 $([$sizes])*), $size>  };
    (mat2x3 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat2x3 $([$sizes])*), $size>  };
    (mat2x4 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat2x4 $([$sizes])*), $size>  };
    (mat3   $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat3   $([$sizes])*), $size>  };
    (mat3x2 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat3x2 $([$sizes])*), $size>  };
    (mat3x3 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat3x3 $([$sizes])*), $size>  };
    (mat3x4 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat3x4 $([$sizes])*), $size>  };
    (mat4   $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat4   $([$sizes])*), $size>  };
    (mat4x2 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat4x2 $([$sizes])*), $size>  };
    (mat4x3 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat4x3 $([$sizes])*), $size>  };
    (mat4x4 $([$sizes:literal])* [$size:literal]) => { $crate::glsl::Array<type_qualifier!(mat4x4 $([$sizes])*), $size>  };
    (sampler1D)                 => { $crate::glsl::Sampler1D         };
    (sampler1DArray)            => { $crate::glsl::Sampler1DArray         };
    (sampler2D)                 => { $crate::glsl::Sampler2D         };
    (sampler2DArray)            => { $crate::glsl::Sampler2DArray         };
    (sampler2DMS)               => { $crate::glsl::Sampler2DMS         };
    (sampler2DMSArray)          => { $crate::glsl::Sampler2DMSArray         };
    (sampler2DRect)             => { $crate::glsl::Sampler2DRect         };
    (sampler3D)                 => { $crate::glsl::Sampler3D         };
    (samplerCube)               => { $crate::glsl::SamplerCube         };
    (samplerBuffer)             => { $crate::glsl::SamplerBuffer         };
    (isampler1D)                => { $crate::glsl::ISampler1D         };
    (isampler1DArray)           => { $crate::glsl::ISampler1DArray         };
    (isampler2D)                => { $crate::glsl::ISampler2D         };
    (isampler2DArray)           => { $crate::glsl::ISampler2DArray         };
    (isampler2DMS)              => { $crate::glsl::ISampler2DMS         };
    (isampler2DMSArray)         => { $crate::glsl::ISampler2DMSArray         };
    (isampler2DRect)            => { $crate::glsl::ISampler2DRect         };
    (isampler3D)                => { $crate::glsl::ISampler3D         };
    (isamplerCube)              => { $crate::glsl::ISamplerCube         };
    (isamplerCubeArray)         => { $crate::glsl::ISamplerCubeArray         };
    (isamplerBuffer)            => { $crate::glsl::iSamplerBuffer         };
    (usampler1D)                => { $crate::glsl::USampler1D         };
    (usampler1DArray)           => { $crate::glsl::USampler1DArray         };
    (usampler2D)                => { $crate::glsl::USampler2D         };
    (usampler2DArray)           => { $crate::glsl::USampler2DArray         };
    (usampler2DMS)              => { $crate::glsl::USampler2DMS         };
    (usampler2DMSArray)         => { $crate::glsl::USampler2DMSArray         };
    (usampler2DRect)            => { $crate::glsl::USampler2DRect         };
    (usampler3D)                => { $crate::glsl::USampler3D         };
    (usamplerCube)              => { $crate::glsl::USamplerCube         };
    (usamplerCubeArray)         => { $crate::glsl::USamplerCubeArray         };
    (usamplerBuffer)            => { $crate::glsl::USamplerBuffer         };
}

#[macro_export]
macro_rules! Glsl {
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*;) => {
        ((), $crate::glsl::variable::Variable::<
            $crate::storage_qualifier!($storage),
            $crate::layout_qualifier!($qualifier = $value),
            $crate::type_qualifier!($type $([$size])*)
        >
    )
    };
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);* ;) => {
        $crate::Glsl! {
            @ ((), $crate::glsl::variable::Variable::<
                $crate::storage_qualifier!($storage),
                $crate::layout_qualifier!($qualifier = $value),
                $crate::type_qualifier!($type $([$size])*)
            >)
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);* 
        }
    };
    (@ $acc:ty => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*) => {
        ($acc, $crate::glsl::variable::Variable::<
            $crate::storage_qualifier!($storage),
            $crate::layout_qualifier!($qualifier = $value),
            $crate::type_qualifier!($type $([$size])*)
        >)
    };
    (@ $acc: ty => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);*) => {
        $crate::Glsl! {
            @ ($acc, $crate::glsl::variable::Variable::<
                $crate::storage_qualifier!($storage),
                $crate::layout_qualifier!($qualifier = $value),
                $crate::type_qualifier!($type $([$size])*)
            >)
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);*
        }
    };
}

#[macro_export]
macro_rules! glsl {
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*;) => {
        $crate::constraint::ValidExt::validated(
            ((), $crate::glsl::variable::Variable::<
                $crate::storage_qualifier!($storage),
                $crate::layout_qualifier!($qualifier = $value),
                $crate::type_qualifier!($type $([$size])*)
            >::default())
        )
    };
    (layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);* ;) => {
        $crate::glsl! {
            @
            $crate::constraint::ValidExt::validated(
                ((), $crate::glsl::variable::Variable::<
                    $crate::storage_qualifier!($storage),
                    $crate::layout_qualifier!($qualifier = $value),
                    $crate::type_qualifier!($type $([$size])*)
                >::default())
            )
            =>
            $(layout($qualifiers = $values) $storages $types $([$sizes])*);*
        }
    };
    (@ $acc:expr => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*) => {
        $crate::constraint::ValidExt::validated(
            ($acc, $crate::glsl::variable::Variable::<
                $crate::storage_qualifier!($storage),
                $crate::layout_qualifier!($qualifier = $value),
                $crate::type_qualifier!($type $([$size])*)
            >::default())
        )
    };
    (@ $acc:expr => layout($qualifier:ident = $value:literal) $storage:ident $type:ident $([$size:literal])*; $(layout($qualifiers:ident = $values:literal) $storages:ident $types:ident $([$sizes:literal])*);*) => {
        $crate::glsl! {
            @
            $crate::constraint::ValidExt::validated(
                ($acc, $crate::glsl::variable::Variable::<
                    $crate::storage_qualifier!($storage),
                    $crate::layout_qualifier!($qualifier = $value),
                    $crate::type_qualifier!($type $([$size])*)
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
        $crate::glsl! {
            layout($qualifier = $value) in $type;
        }
    };
    (layout ($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        $crate::glsl! {
            layout ($qualifier = $value) in $type;
            $(layout ($qualifiers = $values) in $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Inputs {
    (layout ($qualifier:ident = $value:literal) $type:ident;) => {
        $crate::Glsl! {
            layout($qualifier = $value) in $type;
        }
    };
    (layout ($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        $crate::Glsl! {
            layout($qualifier = $value) in $type;
            $(layout ($qualifiers = $values) in $types);* ;
        }
    };
}

#[macro_export]
macro_rules! outputs {
    (layout ($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        $crate::glsl! {
            layout($qualifier = $value) out $type;
        }
    };
    (layout ($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        $crate::glsl! {
            layout ($qualifier = $value) out $type;
            $(layout($qualifiers = $values) out $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Outputs {
    (layout($qualifier:ident = $value:literal) $type:ident ;) => {
        $crate::Glsl! {
            layout($qualifier = $value) out $type;
        }
    };
    (layout($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        $crate::Glsl! {
            layout ($qualifier = $value) out $type;
            $(layout($qualifiers = $values) out $types);* ;
        }
    };
}

#[macro_export]
macro_rules! uniforms {
    (layout($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        $crate::glsl! {
            layout($qualifier = $value) uniform $type;
        }
    };
    (layout($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        $crate::glsl! {
            layout($qualifier = $value) uniform $type;
            $(layout($qualifiers = $values) uniform $types);* ;
        }
    };
}

#[macro_export]
macro_rules! Uniforms {
    (layout($qualifier:ident = $value:literal) $type:ident $(;)?) => {
        $crate::Glsl! {
            layout($qualifier = $value) uniform $type;
        }
    };
    (layout($qualifier:ident = $value:literal) $type:ident; $(layout ($qualifiers:ident = $values:literal) $types:ident);* ;) => {
        $crate::Glsl! {
            layout($qualifier = $value) uniform $type;
            $(layout($qualifiers = $values) uniform $types);* ;
        }
    };
}

/// Unpack bindings into separate variables
#[macro_export]
macro_rules! vars {
    ($ident: ident $(,)?) => {
        ((), $ident)
    };
    ($ident: ident, $($idents: tt),* $(,)?) => {
        $crate::glsl::variable::vars!(@ ((), $ident), $($idents),*)
    };
    (@ $acc: tt, $ident: tt $(,)?) => {
        ($acc, $ident)
    };
    (@ $acc: tt, $ident: ident, $($idents: tt),* $(,)?) => {
        $crate::glsl::variable::vars!(@ ($acc, $ident), $($idents),*)
    };
}

pub use vars;
pub use uniforms;
pub use Uniforms;
pub use inputs;
pub use Inputs;
pub use outputs;
pub use Outputs;
pub use Glsl;

