use std::marker::PhantomData;

use super::super::shader::target;
use super::builder::Builder;
use crate::builder;
use crate::hlist::{self, indexed, lhlist, rhlist};
use crate::{glsl, prelude::HList};
use glsl::binding::{Uniform, UniformBinding, Validated};
use glsl::location;

pub mod collections {
    use super::marker;

    // typical contents of such module / interface come with
    // - Trait for single HList item
    // - Trait for collection of items
    // - Facade wrapper around collection

    /// Facade that provides operations on `Definitions` HLists.
    pub struct Definitions<US>(US)
    where
        US: marker::Definitions;

    // this is analogous to impl Definitions
}

pub mod marker {
    use crate::hlist::lhlist::{Find, Invert};
    use crate::hlist::rhlist::Append;
    use crate::{glsl, hlist};

    use glsl::binding::{Inline, Phantom, UniformBinding, Validated};

    pub trait Definitions: glsl::Uniforms {}

    impl Definitions for () {}

    /// HList of validated bindings to uniforms with inline storage.
    impl<H, const LOCATION: usize, GLSLU> Definitions
        for (H, UniformBinding<GLSLU, LOCATION, Validated, Inline>)
    where
        H: Definitions,
        GLSLU: glsl::Uniform,
    {
    }

    pub trait Declarations<FD>: Clone
    where
        FD: hlist::FoldDirection,
    {
    }

    impl<FD> Declarations<FD> for () where FD: hlist::FoldDirection {}

    pub trait LDeclarations: Declarations<hlist::Left> {}

    pub trait RDeclarations: Declarations<hlist::Right> {}

    impl LDeclarations for () {}
    /// Left folded Declarations
    impl<H, U, const LOCATION: usize> Declarations<hlist::Left> for (H, Declaration<U, LOCATION>)
    where
        H: Declarations<hlist::Left>,
        U: glsl::Uniform,
    {
    }

    impl<H, U, const LOCATION: usize> LDeclarations for (H, Declaration<U, LOCATION>)
    where
        U: glsl::Uniform,
        H: LDeclarations,
    {
    }

    /// Right folded Declarations
    impl<T, U, const LOCATION: usize> Declarations<hlist::Right> for (Declaration<U, LOCATION>, T)
    where
        T: Declarations<hlist::Right>,
        U: glsl::Uniform,
    {
    }

    impl RDeclarations for () {}
    impl<T, U, const LOCATION: usize> RDeclarations for (Declaration<U, LOCATION>, T)
    where
        T: Declarations<hlist::Right>,
        U: glsl::Uniform,
    {
    }
}

pub fn define<const LOCATION: usize, GLU, GLSLU>(
    _: &UniformBinding<GLSLU, LOCATION>,
    uniform: GLU,
) -> Definitions<(DUS, UniformBinding<GLU, GLSLU, LOCATION, Inline>)>
where
    GLSLU: glsl::Uniform<Layout = GLU::Layout>,
    GLU: glsl::compatible::Compatible<GLSLU>,
{
}

pub fn uniform<GLU, GLSLU, const LOCATION: usize>(
    binding: &UniformBinding<GLSLU, LOCATION>,
    uniform: &GLU,
) where
    GLU: glsl::compatible::Compatible<GLSLU>,
    GLSLU: glsl::Uniform<Layout = GLU::Layout>,
    GLSLU: glsl::uniform::ops::Set,
{
}

#[derive(Clone)]
pub struct UniformBinder<DUS, UUS>
where
    DUS: marker::Definitions,
    UUS: marker::Declarations<hlist::Right>,
{
    pub(super) definitions: DUS,
}

impl Default for UniformBinder<(), ()> {
    fn default() -> Self {
        Self {
            definitions: Default::default(),
            declarations: Default::default(),
        }
    }
}

impl<DUS, UUS> UniformBinder<DUS, UUS>
where
    DUS: glsl::Uniforms,
    UUS: marker::RDeclarations,
{
    pub fn new(definitions: Definitions<DUS>) -> Self {
        Self {
            definitions,
            declarations: Declarations::default(),
        }
    }
}

impl<DUS> UniformBinder<DUS, ()>
where
    DUS: marker::Definitions,
{
    /// Definition a new uniform with specified index
    pub fn define<GLU, GLSLU, const LOCATION: usize>(
        self,
        binding: &UniformBinding<GLSLU, LOCATION>,
        uniform: GLU,
    ) -> UniformBinder<(DUS, Definition<GLU, GLSLU, LOCATION>), ()>
    where
        GLU: glsl::compatible::Compatible<GLSLU>,
        GLSLU: glsl::Uniform<Layout = GLU::Layout>,
    {
        let extended = self.definitions.define(binding, uniform);
        UniformBinder {
            definitions: extended,
        }
    }

    /// Add collection of uniforms
    pub fn add_unmatched<UUS: marker::RDeclarations>(self) -> UniformBinder<DUS, UUS> {
        UniformBinder {
            definitions: self.definitions,
        }
    }
}

impl<DUS, HUUS, TUUS, const LOCATION: usize>
    UniformBinder<DUS, (Declaration<HUUS, LOCATION>, TUUS)>
where
    DUS: marker::Definitions,
    HUUS: glsl::Uniform,
    TUUS: marker::RDeclarations,
    (Declaration<HUUS, LOCATION>, TUUS): marker::RDeclarations,
{
    /// Match current head of unmatched uniform list with uniform definition with given index.
    pub fn bind<GLU, IDX>(self, _: &UniformBinding<HUUS, LOCATION>) -> UniformBinder<DUS, TUUS>
    where
        DUS: hlist::lhlist::Find<Definition<GLU, HUUS, LOCATION>, IDX>,
        IDX: hlist::counters::Index,
    {
        UniformBinder::new(self.definitions)
    }
}

// TODO?: Give access to the underlying HList

// /// Type that implements uniform matching.
// pub struct Matcher<'program, AUS, UUS, MUS, I>
// where
//     AUS: lhlist::Base,
//     UUS: rhlist::Base,
//     MUS: lhlist::Append,
// {
//     available: &'program AUS,
//     unspecified: PhantomData<UUS>,
//     specifided: PhantomData<MUS>,
//     indices: I
// }

// impl<AUS, UUS, MUS, I> builder::private::Sealed for Matcher<'_, AUS, UUS, MUS, I> where AUS: lhlist::Base, UUS: rhlist::Base, MUS: lhlist::Append { }

// /// Implement Buider for final stage of Matcher
// impl<AUS, MUS, I> builder::Builder for Matcher<'_, AUS, (), MUS, I> where AUS: lhlist::Append, MUS: lhlist::Append {
//     type Output = ();

//     fn build(self) -> builder::Completed<Self::Output> {
//         todo!()
//     }
// }

// /// Implement Default for Initial stage of Matcher
// impl<AUS, UUS> Default for Matcher<'_, AUS, UUS, (), ()>
// where
//     AUS: lhlist::Append,
//     UUS: rhlist::Base,
// {
//     fn default() -> Self {
//         Self { available: (), unspecified: PhantomData, specifided: PhantomData, indices: PhantomData }
//     }
// }

// impl<'p, AUS, UUS, MUS, I> Matcher<'p, AUS, UUS, MUS, I> where AUS: lhlist::Base, UUS: rhlist::Base, MUS: lhlist::Append {
//     fn retype<NAUS, NUUS, NMUS>(self) -> Matcher<'p, NAUS, NUUS, NMUS, I>
//     where NAUS: lhlist::Base, NUUS: rhlist::Base, NMUS: lhlist::Append
//     {
//         Matcher {
//             available: self.available,
//             unspecified: PhantomData,
//             specifided: PhantomData,
//             indices: self.indices
//         }
//     }
// }

// impl<'p, AUS, UUH, UUT, MUS, I> Matcher<'p, AUS, (UUH, UUT), MUS, I>
// where
//     AUS: lhlist::Append,
//     UUT: rhlist::Base,
//     MUS: lhlist::Append,
// {
//     // NOTE: I here is incorrect -- this approach is incompatiblew with INDEX based selection, I think?
//     pub fn match_uniform(self, _: &UUH) -> Matcher<'p, AUS, UUT, (MUS, UUT), I> {
//         self.retype()
//     }

//     pub fn match_const<const INDEX: usize, IDX>(self) -> Matcher<'p, AUS, UUT, (MUS, UUT), (I, )>
//     where
//         I: indexed::lhlist::Append<UUH>,
//         IDX: crate::hlist::counters::Index,
//         AUS: indexed::lhlist::Get<indexed::Indexed<INDEX, UUH>>,
//     {
//         self.indices.append_indexed(self.available.get::<INDEX, IDX>())
//     }
// }
