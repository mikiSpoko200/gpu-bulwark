use std::marker::PhantomData;

use super::super::shader::target;
use super::builder::Builder;
use super::UniformDefinition;
use crate::builder;
use crate::hlist::{self, indexed, lhlist, rhlist};
use crate::glsl;
use crate::hlist::HList;
use crate::utils;

use glsl::binding::UniformBinding;
use glsl::location;

pub mod marker {
    use crate::glsl::prelude::marker::storage::Uniform;
    use crate::glsl::binding::{UniformBinding, UniformDefinition};
    use crate::{glsl, hlist, ts};
    use crate::mode;

    pub trait Definitions: glsl::Uniforms { }

    impl Definitions for () { }

    /// HList of validated bindings to uniforms with inline storage.
    impl<H, const LOCATION: usize, U> Definitions for (H, UniformBinding<U, LOCATION>)
    where
        H: Definitions,
        U: glsl::Uniform,
    { }

    /// Collection of `UniformBinding`
    pub trait Declarations { }

    impl Declarations for () { }
    impl<T, U, const LOCATION: usize> Declarations for (UniformBinding<U, LOCATION>, T)
    where
        T: Declarations,
        U: glsl::Uniform,
    { }
}


/// Facade that provides operations on `Definitions` HLists.
#[derive(Default)]
pub struct Definitions<US>(US)
where
    US: marker::Definitions;

impl<US> Definitions<US> 
where
    US: marker::Definitions,
{
    /// Definition a new uniform at location specified.
    pub fn define<U, const LOCATION: usize>(self, binding: &UniformBinding<U, LOCATION>, uniform: U::Layout) -> Definitions<(US, UniformBinding<U, LOCATION>)> 
    where
        U: glsl::bounds::TransparentUniform,
    {
        Definitions::default()
    }
}

/// Provides matching between uniform definitions with declaraions during program building.
#[derive(Clone)]
pub struct Matcher<DUS, UUS>(PhantomData<(DUS, UUS)>)
where
    DUS: marker::Definitions,
    UUS: marker::Declarations,
;

impl Default for Matcher<(), ()> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<DUS, UUS> Matcher<DUS, UUS>
where
    DUS: marker::Definitions,
    UUS: marker::Declarations,
{
    pub(super) fn new(definitions: DUS) -> Self {
        Self(PhantomData)
    }
}

impl<DUS, U, T, const LOCATION: usize> Matcher<DUS, (UniformBinding<U, LOCATION>, T)>
where
    DUS: marker::Definitions,
    U: glsl::Uniform,
    T: marker::Declarations,
{
    /// Match current head of unmatched uniform list with uniform definition with given index.
    pub fn bind<GLU, IDX>(self, _: &UniformBinding<U, LOCATION>) -> Matcher<DUS, T>
    where
        DUS: hlist::lhlist::Find<UniformDefinition<U, LOCATION>, IDX>,
        IDX: hlist::counters::Index,
    {
        Matcher::new(self.definitions)
    }
}
