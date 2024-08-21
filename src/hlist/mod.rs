pub mod counters;
pub mod indexed;

use crate::utils::Disjoint;

#[macro_export]
macro_rules! HList {
    ($ty:ty $(,)?) => { 
        ((), $ty)
    };
    ($ty:ty, $($tail:ty),+ $(,)?) => {
        $crate::HList!(@ ((), $ty) => $($tail),+)
    };
    (@ $acc:ty => $ty:ty) => {
        ($acc, $ty)
    };
    (@ $acc:ty => $ty:ty, $($tail:ty),+) => {
        $crate::HList!(@ ($acc, $ty) => $($tail),+)
    };
}

pub use HList;

mod const_ops {
    use super::*;

    const DEFAULT_BUFFER_SIZE: usize = 512;

    pub const fn buffer_size() -> usize {
        let custom_size = option_env!("CONST_OP_BUFFER_SIZE");
    
        if custom_size.is_none() {
            return DEFAULT_BUFFER_SIZE
        }
    
        let mut res: usize = 0;
        let Some(string) = custom_size else {
            panic!("bytes must be not None");
        };
        let mut bytes = string.as_bytes();
        while let [byte, rest @ ..] = bytes {
            bytes = rest;
            if let b'0'..=b'9' = byte {
                res *= 10;
                res += (*byte - b'0') as usize;
            } else {
                panic!("`CONST_OP_BUFFER_SIZE` was not set to a integer value");
            }
        }
        if res > DEFAULT_BUFFER_SIZE { res } else { DEFAULT_BUFFER_SIZE }
    }

    pub const BUFFER_SIZE: usize = buffer_size();

    pub type Buffer<T> = [T; BUFFER_SIZE];

    const EMPTY: usize = 0;

    const fn fill_buffer<T: Copy>(value: T) -> Buffer<T> {
        [value; BUFFER_SIZE]
    }

    const fn empty_buffer() -> Buffer<usize> {
        fill_buffer(EMPTY)
    }

    const fn all_different(array: Buffer<usize>, len: usize, value: usize) -> Buffer<usize> {
        if len > array.len() {
            panic!(
                concat!(
                    module_path!(), "::BUFFER_SIZE too small, you can increase it by setting `CONST_OP_BUFFER_SIZE` environment variable to a greater value. Current value: "
                )
            );
        }
    
        let mut i = 0;
        let mut new_array = empty_buffer();
        while i < len {
            if array[i] == value as _ {
                panic!("items are overlapping");
            }
            new_array[i] = array[i];
            i += 1;
        }
        new_array[len] = value as _;
        new_array
    }

    use lhlist::Base as _;

    pub trait ConstOp: Sized + lhlist::Base { }

    impl ConstOp for () { }
    impl<H, T> ConstOp for (H, T) where H: ConstOp, T: IndexedItem { }

    pub trait AllDifferent: ConstOp + lhlist::Base {
        const MEMORY: Buffer<usize>;
    }

    impl AllDifferent for () {
        const MEMORY: Buffer<usize> = [0; BUFFER_SIZE];
    }

    impl<H, T> AllDifferent for (H, T)
    where
        H: AllDifferent,
        T: IndexedItem,
    {
        const MEMORY: Buffer<usize> = all_different(H::MEMORY, Self::LENGTH - 1, T::INDEX);
    }
}

pub trait IndexedItem {
    const INDEX: usize;
}

pub trait Indexed {
    const CURRENT_ELEMENT_INDEX: usize;
}

impl<E> Indexed for ((), E) where E: IndexedItem {
    const CURRENT_ELEMENT_INDEX: usize = E::INDEX;
}
impl<H, E, T> Indexed for ((H, E), T) where H: Indexed, E: IndexedItem, T: IndexedItem {
    const CURRENT_ELEMENT_INDEX: usize = T::INDEX;
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

    /// Prepended LHList with element.
    pub trait Prepend: Base {
        type Prepended<E>: Prepend;

        fn prepend<E>(self, elem: E) -> Self::Prepended<E>;
    }

    /// Base case
    impl Prepend for () {
        type Prepended<E> = ((), E);

        fn prepend<E>(self, elem: E) -> Self::Prepended<E> {
            (self, elem)
        }
    }

    /// Inductive step
    impl<H: Prepend, T> Prepend for (H, T) {
        type Prepended<E> = (<H as Prepend>::Prepended<E>, T);

        fn prepend<E>(self, elem: E) -> Self::Prepended<E> {
            let (head, tail) = self;
            (head.prepend(elem), tail)
        }
    }

    // --------==========[ Merge ]==========--------

    pub trait Order { }
    pub struct Front;
    impl Order for Front { }
    pub struct Back;
    impl Order for Back { }

    /// Merge two hlists into one by inserting elements from current list to either front or back of second list.
    pub trait Concatenate<Other> {
        type Concatenated: Base;

        fn concatenate(self, other: Other) -> Self::Concatenated;
    }

    impl Concatenate<()> for () {
        type Concatenated = ();

        fn concatenate(self, _: ()) -> Self::Concatenated {
            ()
        }
    }

    impl<H, T> Concatenate<(H, T)> for ()
    where
        H: Base,
    {
        type Concatenated = (H, T);

        fn concatenate(self, other: (H, T)) -> Self::Concatenated {
            other
        }
    }

    impl<H, T> Concatenate<()> for (H, T)
    where
        H: Base,
    {
        type Concatenated = (H, T);

        fn concatenate(self, _: ()) -> Self::Concatenated {
            self
        }
    }

    impl<CH, CT, T> Concatenate<((), T)> for (CH, CT)
    where
        CH: Base,
    {
        type Concatenated = ((CH, CT), T);

        fn concatenate(self, other: ((), T)) -> Self::Concatenated {
            (self, other.1)
        }
    }

    impl<SH, ST, OH, OE, OT> Concatenate<((OH, OE), OT)> for (SH, ST)
    where
        SH: Base,
        OH: Base,
        Self: Concatenate<(OH, OE)>,
    {
        type Concatenated = (<(SH, ST) as Concatenate<(OH, OE)>>::Concatenated, OT);

        fn concatenate(self, (head, tail): ((OH, OE), OT)) -> Self::Concatenated {
            (
                <Self as Concatenate<(OH, OE)>>::concatenate(self, head),
                tail,
            )
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
        (H, E): First,
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
        type Inverted: super::rhlist::Append;

        fn invert(self) -> Self::Inverted;
    }

    /// Base case
    impl Invert for () {
        type Inverted = ();

        fn invert(self) -> Self::Inverted {
            self
        }
    }

    impl<H: Invert, E> Invert for (H, E)
    where
        H: Invert,
        <H::Inverted as super::rhlist::Append>::Appended<E>: super::rhlist::Append,
    {
        type Inverted = <H::Inverted as super::rhlist::Append>::Appended<E>;

        fn invert(self) -> Self::Inverted {
            let (head, tail) = self;
            super::rhlist::Append::append(head.invert(), tail)
        }
    }

    // --------==========[ HList Reversion ]==========--------

    /// Reverse LHList
    pub trait Reverse: Base {
        type Reversed: Prepend;

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
    impl<H: Reverse, E> Reverse for (H, E) {
        type Reversed = <H::Reversed as Prepend>::Prepended<E>;

        fn reverse(self) -> Self::Reversed {
            let (head, elem) = self;
            head.reverse().prepend(elem)
        }
    }

    // --------==========[ HList Selectors ]==========--------

    pub trait Find<Needle, I>: Base
    where
        I: counters::Index,
    {
        fn get(&self) -> &Needle;

        fn get_mut(&mut self) -> &mut Needle;
    }

    impl<H: Base, Needle> Find<Needle, counters::Zero> for (H, Needle) {
        fn get(&self) -> &Needle {
            &self.1
        }

        fn get_mut(&mut self) -> &mut Needle {
            &mut self.1
        }
    }

    impl<H, T, Needle, I> Find<Needle, counters::Successor<I>> for (H, T)
    where
        H: Find<Needle, I>,
        I: counters::Index,
    {
        fn get(&self) -> &Needle {
            self.0.get()
        }

        fn get_mut(&mut self) -> &mut Needle {
            self.0.get_mut()
        }
    }

    // --------==========[ HList Tail ]==========--------

    pub trait Tail: Base {
        type Tail: Base;

        fn tail(self) -> Self::Tail;
    }

    impl Tail for () {
        type Tail = ();
    
        fn tail(self) -> Self::Tail {
            ()
        }
    }

    impl<T: Base> Tail for ((), T) {
        type Tail = ();
    
        fn tail(self) -> Self::Tail {
            ()
        }
    }

    impl<H, E, T> Tail for ((H, E), T)
    where
        H: Tail,
        (H, E): Tail,
    {
        type Tail = (<(H, E) as Tail>::Tail, T);
    
        fn tail(self) -> Self::Tail {
            (self.0.tail(), self.1)
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
    pub trait Append: Base {
        type Appended<E>: Base;

        fn append<E>(self, elem: E) -> Self::Appended<E>;
    }

    /// Base case
    impl Append for () {
        type Appended<E> = (E, ());

        fn append<E>(self, elem: E) -> Self::Appended<E> {
            (elem, ())
        }
    }

    /// Inductive step
    impl<H, T: Append> Append for (H, T) {
        type Appended<E> = (H, <T as Append>::Appended<E>);

        fn append<E>(self, elem: E) -> Self::Appended<E> {
            let (head, tail) = self;
            (head, tail.append(elem))
        }
    }

    // --------==========[ Prepend ]==========--------

    /// Prepended RHList with element.
    pub trait Prepend<E>: Base {
        type Prepended: Base;

        fn prepend(self, elem: E) -> Self::Prepended;
    }

    /// Base case
    impl<E> Prepend<E> for () {
        type Prepended = (E, ());

        fn prepend(self, elem: E) -> Self::Prepended {
            (elem, self)
        }
    }

    /// Inductive step
    impl<E, H, T: Prepend<E>> Prepend<E> for (H, T) {
        type Prepended = (E, (H, T));

        fn prepend(self, elem: E) -> Self::Prepended {
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
        T::Inverted: super::lhlist::Prepend,
    {
        type Inverted = <T::Inverted as super::lhlist::Prepend>::Prepended<E>;

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
        T::Reversed: Append,
    {
        type Reversed = <T::Reversed as Append>::Appended<E>;

        fn reverse(self) -> Self::Reversed {
            let (elem, tail) = self;
            tail.reverse().append(elem)
        }
    }

    // --------==========[ HList Selectors ]==========--------

    /// Selection for RHList
    pub trait Selector<Needle, I>: Base
    where
        I: counters::Index,
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
        I: counters::Index,
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

pub trait FoldDirection {}

pub struct Left;
impl FoldDirection for Left {}

pub struct Right;
impl FoldDirection for Right {}

// TODO: impl for Nil and RCons / LCons ???
impl Disjoint for Left {
    type Discriminant = Self;
}

impl Disjoint for Right {
    type Discriminant = Self;
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
    //     type Prepended<E> = T::Prepended<E>;
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
    //     type Prepended<E> = T::Prepended<E>;
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
