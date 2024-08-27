use crate::ext;
use crate::ffi;
use crate::md;
use crate::prelude::internal::*;

use crate::gl;
use crate::glsl;
use crate::ts;

use crate::hlist::HList;
use crate::hlist::{self, indexed, lhlist, rhlist};
use gl::shader;
use glsl::variable::{TransparentUniformVariable, UniformDefinition};
use ffi::FFIExt;

pub mod bounds {
    use super::*;
    use crate::md;
    use lhlist::Tail;

    #[hi::marker]
    pub trait Declarations { }

    impl Declarations for () {}

    impl<H, U, const LOCATION: usize> Declarations for (H, TransparentUniformVariable<U, LOCATION>)
    where
        H: Declarations,
        U: glsl::Uniform,
    { }

    pub trait Definitions {
        type AsDeclarations: Declarations;
    }

    impl Definitions for () {
        type AsDeclarations = ();
    }

    impl<H, U, T, const LOCATION: usize> Definitions for (H, Definition<'_, U, T, LOCATION>)
    where
        H: Definitions,
        U: glsl::uniform::bounds::TransparentUniform,
        T: glsl::Compatible<U>,
    {
        type AsDeclarations = (H::AsDeclarations, TransparentUniformVariable<U, LOCATION>);
    }
}

/// Typing information for uniform declaration.
#[derive(Clone, Copy)]
pub struct Declaration<U, const LOCATION: usize>(PhantomData<U>)
where
    U: glsl::Uniform;

impl<U, const LOCATION: usize> From<&'_ TransparentUniformVariable<U, LOCATION>> for Declaration<U, LOCATION>
where
    U: glsl::Uniform,
{
    fn from(value: &TransparentUniformVariable<U, LOCATION>) -> Self {
        Self(PhantomData)
    }
}

/// Facade that provides operations on `Declarations` HLists.
#[derive(Copy)]
pub struct Declarations<M, Decls>(pub PhantomData<(M, Decls)>)
where
    M: ts::Mutability,
    Decls: bounds::Declarations;

impl<M, Decls> Clone for Declarations<M, Decls>
where
    M: ts::Mutability,
    Decls: bounds::Declarations,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<Decls> Declarations<ts::Mutable, Decls>
where
    Decls: bounds::Declarations,
{
    pub(crate) fn default() -> Self {
        Self(Default::default())
    }
}

impl Default for Declarations<ts::Immutable, ()> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl From<()> for Declarations<ts::Mutable, ()> {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl<H, Decls, U, const LOCATION: usize> From<&'_ (H, TransparentUniformVariable<U, LOCATION>)> for Declarations<ts::Mutable, (Decls, TransparentUniformVariable<U, LOCATION>)>
where
    Decls: bounds::Declarations,
    Declarations<ts::Mutable, Decls>: From<H>,
    U: glsl::Uniform,
{
    fn from((head, var): &(H, TransparentUniformVariable<U, LOCATION>)) -> Self {
        Declarations(PhantomData)
    }
}


impl<Decls> Declarations<ts::Mutable, Decls>
where
    Decls: bounds::Declarations,
{
    /// Declare a new uniform at specified location.
    pub fn declare<U, const LOCATION: usize>(self, _: Declaration<U, LOCATION>) -> Declarations<ts::Mutable, (Decls, TransparentUniformVariable<U, LOCATION>)>
    where
        U: glsl::Uniform,
    {
        Declarations::default()
    }

    pub(super) fn into_immutable(self) -> Declarations<ts::Immutable, Decls> {
        Declarations(PhantomData)
    }
}

impl<H, U, const LOCATION: usize> Declarations<ts::Immutable, (H, TransparentUniformVariable<U, LOCATION>)>
where
    H: bounds::Declarations,
    U: glsl::Uniform,
{
    pub(super) fn bind(self, _: &TransparentUniformVariable<U, LOCATION>) -> Declarations<ts::Immutable, H> {
        Declarations(PhantomData)
    }
}

#[repr(transparent)]
pub struct Definition<'a, U, T, const LOCATION: usize>(pub(in crate::gl) &'a T, PhantomData<U>) where U: glsl::uniform::bounds::TransparentUniform, T: glsl::Compatible<U>;

impl<'a, U, T, const LOCATION: usize> Definition<'a, U, T, LOCATION>
where 
    U: glsl::uniform::bounds::TransparentUniform,
    T: glsl::Compatible<U>,
{
    pub(in crate::gl) fn new(value: &'a T) -> Self {
        Self(value, PhantomData)
    }
}

/// Facade that provides operations on `Definitions` HLists.
pub struct Definitions<Unis>(pub Unis)
where
    Unis: bounds::Definitions;

impl Definitions<()> {
    pub fn default() -> Self {
        Self(())
    }
}

impl<Unis> Definitions<Unis>
where
    Unis: bounds::Definitions,
{
    /// Add definition for a new transparent uniform.
    pub fn define<'defs, U, T, const LOCATION: usize>(self, _: &'_ TransparentUniformVariable<U, LOCATION>, uniform: &'defs T) -> Definitions<(Unis, Definition<'defs, U, T, LOCATION>)>
    where
        U: glsl::bounds::TransparentUniform,
        T: glsl::Compatible<U>,
    {
        Definitions((self.0, Definition::new(uniform)))
    }
}

/// Provides matching between uniform definitions with declarations during program building.
pub struct Matcher<Defs, Decls>
where
    Defs: bounds::Definitions,
    Decls: bounds::Declarations,
{
    pub(super) definitions: Definitions<Defs>,
    pub(super) declarations: Declarations<ts::Immutable, Decls>,
}

impl Matcher<(), ()> {
    pub(super) fn set_definitions<Defs>(definitions: Definitions<Defs>) -> Matcher<Defs, ()>
    where
        Defs: bounds::Definitions,
    {
        Matcher {
            definitions,
            declarations: Declarations::<ts::Immutable, _>::default(),
        }
    }
}

impl<'defs, Defs> Matcher<Defs, ()>
where
    Defs: bounds::Definitions,
{
    /// Create new Matcher with given set of uniform definitions.
    pub(super) fn new(definitions: Definitions<Defs>) -> Self {
        Matcher {
            definitions,
            declarations: Declarations::<ts::Immutable, _>::default(),
        }
    }

    /// Set new declarations.
    pub(super) fn set_declarations<Decls>(self, decls: Declarations<ts::Mutable, Decls>) -> Matcher<Defs, Decls>
    where
        Decls: bounds::Declarations,
    {
        Matcher {
            definitions: self.definitions,
            declarations: decls.into_immutable(),
        }
    }
}

impl<'defs, Defs, H, U, const LOCATION: usize> Matcher<Defs, (H, TransparentUniformVariable<U, LOCATION>)>
where
    Defs: bounds::Definitions + 'defs,
    H: bounds::Declarations,
    U: glsl::uniform::bounds::TransparentUniform,
{
    /// Match current head of unmatched uniform list with uniform definition with given index.
    pub fn bind<IDX, T>(self, var: &TransparentUniformVariable<U, LOCATION>) -> Matcher<Defs, H>
    where
        T: glsl::Compatible<U> + 'defs,
        Defs: hlist::lhlist::Find<Definition<'defs, U, T, LOCATION>, IDX>,
        IDX: hlist::counters::Index,
    {
        Matcher {
            definitions: self.definitions,
            declarations: self.declarations.bind(var),
        }
    }
}
