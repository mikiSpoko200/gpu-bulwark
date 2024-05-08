use super::lhlist as unordered_lhlist;
use super::rhlist as unordered_rhlist;

// TODO: Is Base bound sound?

// --------==========[ Indexed HLists ]==========--------

pub struct Indexed<const INDEX: usize, T>(T);

impl<const INDEX: usize, T> Indexed<INDEX, T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

pub mod lhlist {
    use frunk::hlist::Selector;

    use super::*;

    // TODO: Add Factory functions

    // FIXME: Should we use `unordered_lhlist::Base` here?
    pub trait Base: unordered_lhlist::Base { }
    
    impl Base for () { }
    impl<const INDEX: usize, H: Base, E> Base for (H, Indexed<INDEX, E>) { }


    // --------==========[ Index HList Empty ]==========--------
    
    pub trait Empty: Base {
        fn default() -> Self;
    
        fn appned_indexed<E>(self, value: E) -> (Self, Indexed<0, E>);
    
        fn new<E>(value: E) -> (Self, Indexed<0, E>);
    }
    
    impl Empty for () {
        fn default() -> Self {
            ()
        }
    
        fn new<E>(value: E) -> (Self, Indexed<0, E>) {
            ((), Indexed::new(value))
        }
    
        fn appned_indexed<E>(self, value: E) -> (Self, Indexed<0, E>) {
            <Self as Empty>::new(value)
        }
    }
    
    #[allow(unused)]
    pub fn new() -> impl Empty {
        <() as Empty>::default()
    }


    // --------==========[ Index HList Append ]==========--------
    
    /// Appending new Indexed Element to LHList
    pub trait Append<E>: Base {
        fn append_indexed<const INDEX: usize>(self, value: E) -> (Self, Indexed<INDEX, E>);
    }
    
    /// Inductive step
    impl<H, T, E> Append<E> for (H, T)
    where
        (H, T): Base
    {
        fn append_indexed<const INDEX: usize>(self, value: E) -> (Self, Indexed<INDEX, E>) {
            (self, Indexed::new(value))
        }
    }

    pub trait Get<E>: Base {
        type Output<const INDEX: usize>;

        fn get<const INDEX: usize, IDX: crate::hlist::counters::Index>(&self) -> &Self::Output<INDEX>
        where
            Self: Selector<Self::Output<INDEX>, IDX>
        ;

        fn get_mut<const INDEX: usize, IDX: crate::hlist::counters::Index>(&mut self) -> &Self::Output<INDEX>
        where
            Self: Selector<Self::Output<INDEX>, IDX>
        ;
    }

    impl<H, T, E> Get<E> for (H, T)
    where
        (H, T): Base
    {
        type Output<const INDEX: usize> = Indexed<INDEX, E>;
    
        fn get<const INDEX: usize, IDX: crate::hlist::counters::Index>(&self) -> &Self::Output<INDEX>
        where
            (H, T): Selector<Self::Output<INDEX>, IDX>
        {
            <Self as Selector<Self::Output<INDEX>, IDX>>::get(self)
        }
        
        fn get_mut<const INDEX: usize, IDX: crate::hlist::counters::Index>(&mut self) -> &Self::Output<INDEX>
        where
            Self: Selector<Self::Output<INDEX>, IDX>
        {
            <Self as Selector<Self::Output<INDEX>, IDX>>::get_mut(self)
        }
    }
}

pub mod rhlist {
    use super::*;

    // TODO: Add Factory functions

    // FIXME: Should we use `unordered_rhlist::Base` here?
    pub trait Base: unordered_rhlist::Base { }
    
    impl Base for () { }
    impl<const INDEX: usize, E, T: Base> Base for (Indexed<INDEX, E>, T) { }


    pub trait Empty: Base {
        fn default() -> Self;

        fn prepend_indexed<E>(self, value: E) -> (Indexed<0, E>, Self);

        fn new<E>(value: E) -> (Indexed<0, E>, Self);
    }

    impl Empty for () {
        fn default() -> Self {
            ()
        }

        fn new<E>(value: E) -> (Indexed<0, E>, Self) {
            (Indexed::new(value), ())
        }

        fn prepend_indexed<E>(self, value: E) -> (Indexed<0, E>, Self) {
            <Self as Empty>::new(value)
        }
    }

    #[allow(unused)]
    pub fn new() -> impl Empty {
        <() as Empty>::default()
    }


    // --------==========[ Index HList Append ]==========--------
    
    pub trait Append<E>: Base {
        fn append_indexed<const INDEX: usize>(self, value: E) -> (Indexed<INDEX, E>, Self);
    }
    
    impl<H, T, E> Append<E> for (H, T)
    where
        (H, T): Base
    {
        fn append_indexed<const INDEX: usize>(self, value: E) -> (Indexed<INDEX, E>, Self) {
            (Indexed::new(value), self)
        }
    }
}
