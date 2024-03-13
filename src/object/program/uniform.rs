//! This module provides specialization of HLists for Program Uniforms.

use std::marker::PhantomData;

use crate::target::shader;
use crate::{prelude::HList, glsl};
use crate::hlist::{lhlist as lhlist, rhlist as rhlist, indexed};
use crate::builder;

pub unsafe trait Uniform { }

pub struct Definitions<const INDEX: usize, U>(pub U);

/// Internal representation of uniforms
pub struct Declaration<const INDEX: usize, U>(pub U);

pub trait Declarations { }

impl Declarations for () { }
impl<H, T> Declarations for (H, T) { }

/// Type level index
pub struct Index<const VALUE: usize>;

impl<const VALUE: usize> Index<VALUE> {
    pub const VALUE: usize = VALUE;
}

pub struct Uniforms<US: lhlist::Append>(US);

impl AsRef<()> for Uniforms<()> {
    fn as_ref(&self) -> &() {
        &self.0
    }
}

impl<H, T> AsRef<(H, T)> for Uniforms<(H, T)>
where
    (H, T): lhlist::Append
{
    fn as_ref(&self) -> &(H, T) {
        &self.0
    }
}

// TODO: AUS to UniformDecls from ProgramBuilder

/// Type that implements uniform matching.
pub struct Matcher<'program, AUS, UUS, MUS, I>
where
    AUS: lhlist::Base,
    UUS: rhlist::Base,
    MUS: lhlist::Append,
{
    available: &'program AUS,
    unspecified: PhantomData<UUS>,
    specifided: PhantomData<MUS>,
    indices: I
}

impl<AUS, UUS, MUS, I> builder::private::Sealed for Matcher<'_, AUS, UUS, MUS, I> where AUS: lhlist::Base, UUS: rhlist::Base, MUS: lhlist::Append { }

/// Implement Buider for final stage of Matcher
impl<AUS, MUS, I> builder::Builder for Matcher<'_, AUS, (), MUS, I> where AUS: lhlist::Append, MUS: lhlist::Append {
    type Output = ();

    fn build(self) -> builder::Completed<Self::Output> { }
}

/// Implement Default for Initial stage of Matcher
impl<AUS, UUS> Default for Matcher<'_, AUS, UUS, (), ()>
where
    AUS: lhlist::Append,
    UUS: rhlist::Base,
{
    fn default() -> Self {
        Self { available: PhantomData, unspecified: PhantomData, specifided: PhantomData, indices: PhantomData }
    }
}

impl<'p, AUS, UUS, MUS, I> Matcher<'p, AUS, UUS, MUS, I> where AUS: lhlist::Base, UUS: rhlist::Base, MUS: lhlist::Append {
    fn retype<NAUS, NUUS, NMUS>(self) -> Matcher<'p, NAUS, NUUS, NMUS, I>
    where NAUS: lhlist::Base, NUUS: rhlist::Base, NMUS: lhlist::Append
    {
        Matcher {
            available: self.available,
            unspecified: PhantomData,
            specifided: PhantomData,
            indices: self.indices
        }
    }
}

impl<'p, AUS, UUH, UUT, MUS, I> Matcher<'p, AUS, (UUH, UUT), MUS, I> 
where
    AUS: lhlist::Append,
    UUT: rhlist::Base,
    MUS: lhlist::Append,
{
    // NOTE: I here is incorrect -- this approach is incompatiblew with INDEX based selection, I think?
    pub fn match_uniform(self, _: &UUH) -> Matcher<'p, AUS, UUT, (MUS, UUT), I> {
        self.retype()
    }

    pub fn match_const<const INDEX: usize, IDX>(self) -> Matcher<'p, AUS, UUT, (MUS, UUT), (I, )>
    where
        I: indexed::lhlist::Append<UUH>,
        IDX: crate::hlist::counters::Index,
        AUS: indexed::lhlist::Get<indexed::Indexed<INDEX, UUH>>,
    {
        self.indices.append_indexed(self.available.get::<INDEX, IDX>())
    }
}
