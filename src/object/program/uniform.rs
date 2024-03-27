
use std::marker::PhantomData;

use super::super::shader::target;
use crate::{prelude::HList, glsl};
use crate::hlist::{self, indexed, lhlist as lhlist, rhlist as rhlist};
use crate::builder;
use super::builder::Builder;

pub mod marker {
    use crate::hlist;
    use crate::hlist::lhlist::Invert;
    use crate::hlist::rhlist::Append;

    use super::{Definition, Declaration};

    pub unsafe trait Uniform: Clone + Default { }

    macro_rules! impl_uniform {
        ($tt: ty) => {
            unsafe impl Uniform for $tt { }
        };
    }

    impl_uniform!(f32);
    impl_uniform!([f32; 2]);
    impl_uniform!([f32; 3]);
    impl_uniform!([f32; 4]);

    impl_uniform!([[f32; 2]; 2]);
    impl_uniform!([[f32; 2]; 3]);
    impl_uniform!([[f32; 2]; 4]);

    impl_uniform!([[f32; 3]; 2]);
    impl_uniform!([[f32; 3]; 3]);
    impl_uniform!([[f32; 3]; 4]);

    impl_uniform!([[f32; 4]; 2]);
    impl_uniform!([[f32; 4]; 3]);
    impl_uniform!([[f32; 4]; 4]);

    // TODO: impl Uniform

    impl_uniform!(u32);
    impl_uniform!(i32);

    pub trait Definitions: Clone { }

    impl Definitions for () { }
    impl<H, const INDEX: usize, U> Definitions for (H, Definition<INDEX, U>)
    where
        H: Definitions,
        U: Uniform
    {}

    pub trait Declarations<FD>: Clone + Default
    where
        FD: hlist::FoldDirection
    {}

    impl<FD> Declarations<FD> for () where FD: hlist::FoldDirection {}

    pub trait LDeclarations: Declarations<hlist::Left> { }

    pub trait RDeclarations: Declarations<hlist::Right> { }

    impl LDeclarations for () { }
    /// Left folded Declarations
    impl<H, U> Declarations<hlist::Left> for (H, Declaration<U>)
    where
        H: Declarations<hlist::Left>,
        U: Uniform
    {}

    impl<H, U> LDeclarations for (H, Declaration<U>)
    where
        U: Uniform,
        H: LDeclarations,
    {}

    /// Right folded Declarations
    impl<T, U> Declarations<hlist::Right> for (Declaration<U>, T)
    where
        T: Declarations<hlist::Right>,
        U: Uniform
    {}

    impl RDeclarations for () { }
    impl<T, U> RDeclarations for (Declaration<U>, T)
    where
        T: Declarations<hlist::Right>,
        U: Uniform
    {}
}

#[derive(Clone)]
pub struct Definition<const INDEX: usize, U>(pub U) where U: marker::Uniform;

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
    pub fn define<const INDEX: usize, U>(self, u: U) -> Definitions<(DUS, Definition<INDEX, U>)>
    where
        U: marker::Uniform
    {
        Definitions {
            values: (self.values, Definition(u)),
            locations: self.locations,
        }
    }

    pub fn vertex_main<VI, VO, US>(self, vertex: &super::Main<super::Vertex, VI, VO, US>) -> Builder<super::Vertex, VI, VO, DUS, US::Inverted>
    where
        VI: super::parameters::Parameters,
        VO: super::parameters::Parameters,
        US: marker::LDeclarations + lhlist::Invert,
        US::Inverted: marker::RDeclarations
    {
        Builder::<super::Vertex, VI, VO, DUS, ()>::new::<US>(vertex, self)
    }
}

#[derive(Clone)]
pub struct Declaration<U>(PhantomData<U>) where U: marker::Uniform;

impl<U: marker::Uniform> Default for Declaration<U> {
    fn default() -> Self {
        Self(Default::default())
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
    pub fn define<const INDEX: usize, U: marker::Uniform>(self, u: U) -> Uniforms<(DUS, Definition<INDEX, U>), ()> {
        let extended = self.definitions.define(u);
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

impl<DUS, HUUS, TUUS> Uniforms<DUS, (Declaration<HUUS>, TUUS)>
where
    DUS: marker::Definitions,
    HUUS: marker::Uniform,
    TUUS: marker::RDeclarations,
    (Declaration<HUUS>, TUUS): marker::RDeclarations
{
    /// Match current head of unmatched uniform list with uniform definition with given index.
    pub fn match_uniform<const INDEX: usize, IDX>(self) -> Uniforms<DUS, TUUS>
    where
        DUS: hlist::lhlist::Selector<Definition<INDEX, HUUS>, IDX>,
        IDX: hlist::counters::Index,
    {
        let _ = self.definitions.0.get();

        Uniforms {
            definitions: self.definitions,
            declarations: Declarations::default(),
        }
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
