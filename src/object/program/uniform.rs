
use std::marker::PhantomData;

use super::super::shader::target;
use crate::glsl::location;
use crate::{prelude::HList, glsl};
use crate::hlist::{self, indexed, lhlist as lhlist, rhlist as rhlist};
use crate::builder;
use super::builder::Builder;
use glsl::location::{Location, Validated, Unvalidated};

pub mod marker {
    use crate::{glsl, hlist};
    use crate::hlist::lhlist::Invert;
    use crate::hlist::rhlist::Append;

    use super::{Definition, Declaration};

    pub trait Definitions: Clone { }

    impl Definitions for () { }
    impl<H, const LOCATION: usize, GLU, GLSLU> Definitions for (H, Definition<GLU, GLSLU, LOCATION>)
    where
        H: Definitions,
        GLU: Clone,
        GLSLU: glsl::Uniform,
        (GLU, GLSLU): glsl::compatible::Compatible<GLU, GLSLU>
    {}

    pub trait Declarations<FD>: Clone
    where
        FD: hlist::FoldDirection
    {}

    impl<FD> Declarations<FD> for () where FD: hlist::FoldDirection {}

    pub trait LDeclarations: Declarations<hlist::Left> { }

    pub trait RDeclarations: Declarations<hlist::Right> { }

    impl LDeclarations for () { }
    /// Left folded Declarations
    impl<H, U, const LOCATION: usize> Declarations<hlist::Left> for (H, Declaration<U, LOCATION>)
    where
        H: Declarations<hlist::Left>,
        U: glsl::Uniform
    {}

    impl<H, U, const LOCATION: usize> LDeclarations for (H, Declaration<U, LOCATION>)
    where
        U: glsl::Uniform,
        H: LDeclarations,
    {}

    /// Right folded Declarations
    impl<T, U, const LOCATION: usize> Declarations<hlist::Right> for (Declaration<U, LOCATION>, T)
    where
        T: Declarations<hlist::Right>,
        U: glsl::Uniform
    {}

    impl RDeclarations for () { }
    impl<T, U, const LOCATION: usize> RDeclarations for (Declaration<U, LOCATION>, T)
    where
        T: Declarations<hlist::Right>,
        U: glsl::Uniform
    {}
}

#[derive(Clone)]
pub struct Definition<GLU, GLSLU, const LOCATION: usize>(pub GLU, PhantomData<GLSLU>);

#[derive(Clone)]
pub struct Definitions<US>
where
    US: marker::Definitions
{
    pub values: US,
    pub locations: Vec<u32>,
} 

impl Definitions<()> {
    pub fn new() -> Self {
        Self {
            values: (), 
            locations: Vec::new()
        }
    }
}

impl Default for Definitions<()> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DUS> Definitions<DUS>
where
    DUS: marker::Definitions,
{
    pub fn define<const LOCATION: usize, GLU, GLSLU>(self, uniform: GLU, _: &Location<GLSLU, LOCATION, Validated>) -> Definitions<(DUS, Definition<GLU, GLSLU, LOCATION>)>
    where
        GLSLU: glsl::Uniform,
        GLU: Clone,
        (GLU, GLSLU): glsl::compatible::Compatible<GLU, GLSLU>,
    {
        Definitions {
            values: (self.values, Definition(uniform, PhantomData)),
            locations: self.locations,
        }
    }

    // pub fn vertex_main<VI, VO, US>(self, vertex: &super::Main<super::Vertex, VI, VO, US>) -> Builder<super::Vertex, VI, VO, DUS, US::Inverted>
    // where
    //     VI: super::parameters::Parameters,
    //     VO: super::parameters::Parameters,
    //     US: marker::LDeclarations + lhlist::Invert,
    //     US::Inverted: marker::RDeclarations
    // {
    //     self.vertex_main(vertex)
    // }
}

#[derive(Clone)]
pub struct Declaration<U, const LOCATION: usize>(Location<U, LOCATION, Validated>) where U: glsl::Uniform;

impl<U, const LOCATION: usize> Declaration<U, LOCATION>
where
    U: glsl::Uniform
{
    pub const fn new(loaction: Location<U, LOCATION, Validated>) -> Self {
        Self(loaction)
    }
}

#[derive(Clone)]
pub struct Declarations<US>(PhantomData<US>) where US: marker::RDeclarations;

impl<US> Default for Declarations<US>
where
    US: marker::RDeclarations
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub struct Index<const VALUE: usize>;

impl<const VALUE: usize> Index<VALUE> {
    pub const VALUE: usize = VALUE;
}

#[derive(Clone)]
pub struct Uniforms<DUS, UUS>
where
    DUS: marker::Definitions,
    UUS: marker::RDeclarations,
{
    pub(super) definitions: Definitions<DUS>,
    pub(super) declarations: Declarations<UUS>,
}

impl Default for Uniforms<(), ()> {
    fn default() -> Self {
        Self { definitions: Default::default(), declarations: Default::default() }
    }
}

impl<DUS, UUS> Uniforms<DUS, UUS> 
where
    DUS: marker::Definitions,
    UUS: marker::RDeclarations,
{
    pub fn new(definitions: Definitions<DUS>) -> Self {
        Self {
            definitions,
            declarations: Declarations::default(),
        }
    }
}

impl<DUS> Uniforms<DUS, ()>
where
    DUS: marker::Definitions,
{
    /// Definition a new uniform with specified index
    pub fn define<GLU, GLSLU, const LOCATION: usize>(self, uniform: GLU, location: &Location<GLSLU, LOCATION, Validated>) -> Uniforms<(DUS, Definition<GLU, GLSLU, LOCATION>), ()>
    where
        GLSLU: glsl::Uniform,
        GLU: Clone,
        (GLU, GLSLU): glsl::compatible::Compatible<GLU, GLSLU>,
    {
        let extended = self.definitions.define(uniform, location);
        Uniforms {
            declarations: Declarations::default(),
            definitions: extended,
        }
    }
  
    /// Add collection of uniforms 
    pub fn add_unmatched<UUS: marker::RDeclarations>(self) -> Uniforms<DUS, UUS> {
        Uniforms {
            definitions: self.definitions,
            declarations: Declarations::default(),
        }
    }
}

impl<DUS, HUUS, TUUS, const LOCATION: usize> Uniforms<DUS, (Declaration<HUUS, LOCATION>, TUUS)>
where
    DUS: marker::Definitions,
    HUUS: glsl::Uniform,
    TUUS: marker::RDeclarations,
    (Declaration<HUUS, LOCATION>, TUUS): marker::RDeclarations
{
    /// Match current head of unmatched uniform list with uniform definition with given index.
    pub fn match_uniform<GLU, IDX>(self, _: &Location<HUUS, LOCATION, Validated>) -> Uniforms<DUS, TUUS>
    where
        DUS: hlist::lhlist::Selector<Definition<GLU, HUUS, LOCATION>, IDX>,
        IDX: hlist::counters::Index,
    {
        Uniforms::new(self.definitions)
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
