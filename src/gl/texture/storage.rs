use std::marker::PhantomData;


pub mod marker {
    #[hi::marker]
    pub trait Storage { }
}

#[hi::mark(marker::Storage)]
pub enum Immutable { }

#[hi::mark(marker::Storage)]
pub enum Mutable { }

#[hi::mark(marker::Storage)]
pub enum Buffer { }

pub struct Storage<S>(PhantomData<S>)
where
    S: marker::Storage
;

pub trait Allocate {
    // fn storage(&mut self, );
}