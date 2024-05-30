use std::marker::PhantomData;

// --------==========[ Type Level Counters ]==========--------

pub trait Index {
    const INDEX: usize;
}

pub struct Zero;
impl Index for Zero {
    const INDEX: usize = 0;
}

pub struct Successor<I: Index>(PhantomData<I>);
impl<I: Index> Index for Successor<I> {
    const INDEX: usize = I::INDEX + 1;
}
