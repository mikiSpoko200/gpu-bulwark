pub mod counters;
pub mod indexed;

pub trait Disjoint {
    type Discriminant;
}

pub mod lhlist {
    #[allow(unused)]
    use super::{counters, Disjoint, Left};

    
    // --------==========[ Base Traits ]==========--------

    /// Left folded HList
    pub trait Base: Sized {
        const LENGTH: usize;
        
        fn append<T>(self, t: T) -> (Self, T) {
            (self, t)
        }
    }
       
    impl Base for () {
        const LENGTH: usize = 0;
    }
    
    impl<T: Base, H> Base for (T, H) {
        const LENGTH: usize = T::LENGTH + 1;
    }


    // --------==========[ Empty HLists ]==========--------
    
    pub trait Empty: Base {
        fn default() -> Self;
    }
    
    impl Empty for () {
        fn default() -> Self {
            ()
        }
    }


    // --------==========[ Append ]==========--------
    
    /// Append for LHList.
    pub trait Append: Base {
        type Appended<E>: Base;
    
        fn append<E>(self, elem: E) -> Self::Appended<E>;
    }
    
    /// Base case
    impl Append for () {
        type Appended<E> = ((), E);
    
        fn append<E>(self, elem: E) -> Self::Appended<E> {
            ((), elem)
        }
    }
    
    /// Inductive step
    impl<H: Append, T> Append for (H, T) {
        type Appended<E> = ((H, T), E);
    
        fn append<E>(self, elem: E) -> Self::Appended<E> {
            (self, elem)
        }
    }


    // --------==========[ Prepend ]==========--------
    
    /// Preprended LHList with element.
    pub trait Prepend: Base {
        type Preprended<E>: Base;
    
        fn prepend<E>(self, elem: E) -> Self::Preprended<E>;
    }
    
    /// Base case
    impl Prepend for () {
        type Preprended<E> = ((), E);
        
        fn prepend<E>(self, elem: E) -> Self::Preprended<E> {
            (self, elem)
        }
    }
    
    /// Inductive step
    impl<H: Prepend, T> Prepend for (H, T) {
        type Preprended<E> = (<H as Prepend>::Preprended<E>, T);
        
        fn prepend<E>(self, elem: E) -> Self::Preprended<E> {
            let (head, tail) = self;
            (head.prepend(elem), tail)
        }
    }


    // --------==========[ First ]==========--------

    /// Getter for first element
    pub trait First: Base {
        type First;

        fn first(&self) -> &Self::First;
        fn first_mut(&mut self) -> &mut Self::First;
    }

    impl<E> First for ((), E) {
        type First = E;
    
        fn first(&self) -> &Self::First {
            &self.1
        }
    
        fn first_mut(&mut self) -> &mut Self::First {
            &mut self.1
        }
    }
    
    impl<H, E, T> First for ((H, E), T)
    where
        (H, E): First
    {
        type First = <(H, E) as First>::First;
    
        fn first(&self) -> &Self::First {
            self.0.first()
        }
        
        fn first_mut(&mut self) -> &mut Self::First {
            self.0.first_mut()
        }
    }
    
    
    // --------==========[ Last ]==========--------
    
    /// Getter for the last element
    pub trait Last: Base {
        type Last;
    
        fn last(&self) -> &Self::Last;
        fn last_mut(&mut self) -> &mut Self::Last;
    }
    
    impl<H: Base, E> Last for (H, E) {
        type Last = E;
        
        fn last(&self) -> &Self::Last {
            &self.1
        }
        
        fn last_mut(&mut self) -> &mut Self::Last {
            &mut self.1
        }
    }


    // --------==========[ Inversion LHList <-> RHList ]==========--------
    
    /// LHList conversion to RHList
    pub trait Invert: Base {
        type Inverted: super::rhlist::Base;
    
        fn invert(self) -> Self::Inverted;
    }
    
    /// Base case
    impl Invert for () {
        type Inverted = ();
    
        fn invert(self) -> Self::Inverted {
            self
        }
    }
    
    /// Inductive step
    impl<H: Invert, E> Invert for (H, E)
    where
        H::Inverted: super::rhlist::Append<E>
    {
        type Inverted = <H::Inverted as super::rhlist::Append<E>>::Appended;
    
        fn invert(self) -> Self::Inverted {
            let (head, elem) = self;
            <H::Inverted as super::rhlist::Append<E>>::append(head.invert(), elem)
        }
    }


    // --------==========[ HList Reversion ]==========--------

    /// Reverse LHList
    pub trait Reverse: Base {
        type Reversed: Base;

        fn reverse(self) -> Self::Reversed;
    }

    /// Base case
    impl Reverse for () {
        type Reversed = ();

        fn reverse(self) -> Self::Reversed {
            self
        }
    }

    /// Inductive step
    impl<H: Reverse, E> Reverse for (H, E)
    where
        H::Reversed: Prepend
    {
        type Reversed = <H::Reversed as Prepend>::Preprended<E>;
        
        fn reverse(self) -> Self::Reversed {
            let (head, elem) = self;
            head.reverse().prepend(elem)
        }
    }


    // --------==========[ HList Selectors ]==========--------
    
    pub trait Selector<Needle, I>: Base
    where
        I: counters::Index
    {
        fn get(&self) -> &Needle;
    
        fn get_mut(&mut self) ->&mut Needle;
    }
    
    impl<H: Base, Needle> Selector<Needle, counters::Zero> for (H, Needle) {
        fn get(&self) -> &Needle {
            &self.1
        }
    
        fn get_mut(&mut self) -> &mut Needle {
            &mut self.1
        }
    }
    
    impl<H, T, Needle, I> Selector<Needle, counters::Successor<I>> for (H, T)
    where
        H: Selector<Needle, I>,
        I: counters::Index,
    {
        fn get(&self) -> &Needle {
            self.0.get()
        }
    
        fn get_mut(&mut self) -> &mut Needle {
            self.0.get_mut()
        }
    }

}

pub mod rhlist {
    use super::counters;


    // --------==========[ Base Traits ]==========--------
    /// Right folded HList
    pub trait Base: Sized {
        const LENGTH: usize;

        fn prepend<H>(self, h: H) -> (H, Self) {
            (h, self)
        }
    }

    impl Base for () {
        const LENGTH: usize = 0;
    }

    impl<H, T: Base> Base for (H, T) {
        const LENGTH: usize = T::LENGTH + 1;
    }


    // --------==========[ Empty HLists ]==========--------

    pub trait Empty: Base {
        fn default() -> Self;
    }

    impl Empty for () {
        fn default() -> Self {
            ()
        }
    }


    // --------==========[ Append ]==========--------

    /// Append for RHList.
    pub trait Append<E>: Base {
        type Appended: Base;

        fn append(self, elem: E) -> Self::Appended;
    }

    /// Base case
    impl<E> Append<E> for () {
        type Appended = (E, ());

        fn append(self, elem: E) -> Self::Appended {
            (elem, ())
        }
    }

    /// Inductive step
    impl<E, H, T: Append<E>> Append<E> for (H, T) {
        type Appended = (H, <T as Append<E>>::Appended);

        fn append(self, elem: E) -> Self::Appended {
            let (head, tail) = self;
            (head, tail.append(elem))
        }
    }


    // --------==========[ Prepend ]==========--------

    /// Preprended RHList with element.
    pub trait Prepend<E>: Base {
        type Preprended: Base;

        fn prepend(self, elem: E) -> Self::Preprended;
    }

    /// Base case
    impl<E> Prepend<E> for () {
        type Preprended = (E, ());
        
        fn prepend(self, elem: E) -> Self::Preprended {
            (elem, self)
        }
    }

    /// Inductive step
    impl<E, H, T: Prepend<E>> Prepend<E> for (H, T) {
        type Preprended = (E, (H, T));
        
        fn prepend(self, elem: E) -> Self::Preprended {
            (elem, self)
        }
    }


    // --------==========[ First ]==========--------

    pub trait First: Base {
        type First;

        fn first(&self) -> &Self::First;
        fn first_mut(&mut self) -> &mut Self::First;
    }

    impl<E, T: Base> First for (E, T) {
        type First = E;
    
        fn first(&self) -> &Self::First {
            &self.0
        }
    
        fn first_mut(&mut self) -> &mut Self::First {
            &mut self.0
        }
    }


    // --------==========[ Last ]==========--------
    
    /// Getter for last element
    pub trait Last: Base {
        type Last;
    
        fn last(&self) -> &Self::Last;
        fn last_mut(&mut self) -> &mut Self::Last;
    }
    
    /// Base case
    impl<E> Last for (E, ()) {
        type Last = E;
    
        fn last(&self) -> &Self::Last {
            &self.0
        }
        
        fn last_mut(&mut self) -> &mut Self::Last {
            &mut self.0
        }
    }
    
    /// Inductive step
    impl<H, E, T> Last for (H, (E, T))
    where
        (E, T): Last,
    {
        type Last = <(E, T) as Last>::Last;
    
        fn last(&self) -> &Self::Last {
            self.1.last()
        }
        
        fn last_mut(&mut self) -> &mut Self::Last {
            self.1.last_mut()
        }
    }


    // --------==========[ Inversion LHList <-> RHList ]==========--------

    /// RHList conversion to LHList.
    pub trait Invert: Base {
        type Inverted: super::lhlist::Base;

        fn invert(self) -> Self::Inverted;
    }

    /// Base case
    impl Invert for () {
        type Inverted = ();

        fn invert(self) -> Self::Inverted {
            ()
        }
    }

    /// Inductive step
    impl<E, T: Invert> Invert for (E, T)
    where
        T::Inverted: super::lhlist::Prepend
    {
        type Inverted = <T::Inverted as super::lhlist::Prepend>::Preprended<E>;

        fn invert(self) -> Self::Inverted {
            let (elem, tail) = self;
            <T::Inverted as super::lhlist::Prepend>::prepend(tail.invert(), elem)
        }
    }

    
    // --------==========[ HList Reversion ]==========--------

    /// Reverse RHList
    pub trait Reverse: Base {
        type Reversed: Base;

        fn reverse(self) -> Self::Reversed;
    }

    /// Base case
    impl Reverse for () {
        type Reversed = ();
        
        fn reverse(self) -> Self::Reversed {
            self
        }
    }

    /// Inductive step
    impl<E, T: Reverse> Reverse for (E, T)
    where
        T::Reversed: Append<E>,
    {
        type Reversed = <T::Reversed as Append<E>>::Appended;
        
        fn reverse(self) -> Self::Reversed {
            let (elem, tail) = self;
            tail.reverse().append(elem)
        }
    }


    // --------==========[ HList Selectors ]==========--------

    /// Selection for RHList
    pub trait Selector<Needle, I>: Base
    where
        I: counters::Index
    {
        fn get(&self) -> &Needle;

        fn get_mut(&mut self) -> &mut Needle;
    }

    impl<Needle, T: Base> Selector<Needle, counters::Zero> for (Needle, T) {
        fn get(&self) -> &Needle {
            &self.0
        }

        fn get_mut(&mut self) -> &mut Needle {
            &mut self.0
        }
    }

    impl<H, T, Needle, I> Selector<Needle, counters::Successor<I>> for (H, T)
    where
        T: Selector<Needle, I>,
        I: counters::Index
    {
        fn get(&self) -> &Needle {
            self.1.get()
        }

        fn get_mut(&mut self) -> &mut Needle {
            self.1.get_mut()
        }
    }
}


// --------==========[ Unified HList ]==========--------

pub struct Left;
pub struct Right;


// TODO: impl for Nil and RCons / LCons ???
impl Disjoint for Left {
    type Discriminant = Self;
}

impl Disjoint for Right {
    type Discriminant = Self;
}

pub trait HList {
    type Prepended: HList;
    type Appended: HList;

    type Last;
    type First;

    type Inverted: HList;
    type Reversed: HList;

    fn prepend<E>(self, value: E) -> Self::Prepended;
    fn append<E>(self, value: E) -> Self::Appended;

    fn first(&self) -> Self::First;
    fn last(&self) -> Self::Last;

    fn invert(&self) -> Self::Inverted;
    fn reverse(self) -> Self::Reversed;
}

mod private {
    #[allow(unused)]
    use super::*;

    // This delegates to a private helper trait which we can specialize on in stable rust
    // impl<T: Disjoint + HListHelper<T::Discriminant>> HList for T {
    //     type Prepended = T::Prepended;
    //     type Appended = T::Appended;
        
    //     type Last = T::Last;
    //     type First = T::First;
        
    //     type Inverted = T::Inverted;
    //     type Reversed = T::Reversed;
        
    //     fn prepend<E>(self, value: E) -> Self::Prepended {
    //         todo!()
    //     }
        
    //     fn append<E>(self, value: E) -> Self::Appended {
    //         todo!()
    //     }
        
    //     fn first(&self) -> Self::First {
    //         todo!()
    //     }
        
    //     fn last(&self) -> Self::Last {
    //         todo!()
    //     }
        
    //     fn invert(&self) -> Self::Inverted {
    //         todo!()
    //     }
        
    //     fn reverse(self) -> Self::Reversed {
    //         todo!()
    //     }
    //     // TODO: Implement HList interface using HListHelper
    // }
    
    trait HListHelper<Type> {
        type Prepended;
        type Appended;
    
        type Last;
        type First;
    
        type Inverted;
        type Reversed;

        type Selected<N, I>;
    
        fn prepend<E>(self, value: E) -> Self::Prepended;
        fn append<E>(self, value: E) -> Self::Appended;
    
        fn first(&self) -> Self::First;
        fn last(&self) -> Self::Last;
    
        fn invert(&self) -> Self::Inverted;
        fn reverse(self) -> Self::Reversed;

        fn select<N, I>(&self) -> Self::Selected<N, I>;
    }

    // // blanket impl 1
    // impl<T, N, I> HListHelper<super::Left> for T
    // where
    //     T: lhlist::Base
    //     + lhlist::Append
    //     + lhlist::Prepend
    //     + lhlist::First
    //     + lhlist::Last
    //     + lhlist::Invert
    //     + lhlist::Reverse
    //     + lhlist::Selector<N, I>,
    // {
    //     type Prepended<E> = T::Preprended<E>;
    //     type Appended<E> = T::Appended<E>;

    //     type Last = T::Last;
    //     type First = T::First;
        
    //     type Inverted = T::Inverted;
    //     type Reversed = T::Reversed;
        
    //     fn prepend<E>(self, value: E) -> Self::Prepended {
    //         todo!()
    //     }
        
    //     fn append<E>(self, value: E) -> Self::Appended {
    //         todo!()
    //     }
        
    //     fn first(&self) -> Self::First {
    //         todo!()
    //     }
        
    //     fn last(&self) -> Self::Last {
    //         todo!()
    //     }
        
    //     fn invert(&self) -> Self::Inverted {
    //         todo!()
    //     }
        
    //     fn reverse(self) -> Self::Reversed {
    //         todo!()
    //     }

    //     fn select(&self) -> Self::Selected<N, I> {
    //         todo!()
    //     }
    //     // TODO: Concrete impl for LHList
    // }
    
    // // blanket impl 2
    // impl<T, N, I> HListHelper<Right> for T
    // where
    //     T: lhlist::Base
    //     + lhlist::Append
    //     + lhlist::Prepend
    //     + lhlist::First
    //     + lhlist::Last
    //     + lhlist::Invert
    //     + lhlist::Reverse
    //     + lhlist::Selector<N, I>,
    // {
    //     type Prepended<E> = T::Preprended<E>;
    //     type Appended<E> = T::Appended<E>;

    //     type Last = T::Last;
    //     type First = T::First;
        
    //     type Inverted = T::Inverted;
    //     type Reversed = T::Reversed;
        
    //     fn prepend<E>(self, value: E) -> Self::Prepended {
    //         todo!()
    //     }
        
    //     fn append<E>(self, value: E) -> Self::Appended {
    //         todo!()
    //     }
        
    //     fn first(&self) -> Self::First {
    //         todo!()
    //     }
        
    //     fn last(&self) -> Self::Last {
    //         todo!()
    //     }
        
    //     fn invert(&self) -> Self::Inverted {
    //         todo!()
    //     }
        
    //     fn reverse(self) -> Self::Reversed {
    //         todo!()
    //     }
        // TODO: Concrete impl for RHList
    // }
}
