//! Auxiliary Many-modes patten implementations.

/// Many-modes pattern for abstraction over storage
pub trait Storage {
    type Store<T>;
}

#[derive(Clone, Copy, Debug)]
/// Storage
pub struct Phantom;

#[derive(Clone, Copy, Debug)]
pub struct Inline;

impl Storage for Phantom {
    type Store<T> = std::marker::PhantomData<T>;
}

impl Storage for Inline {
    type Store<T> = T;
}

pub trait Selector: Storage {
    type Storage: Storage;
}

pub trait Validation { }
