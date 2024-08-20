use std::marker::PhantomData;

use himark as hi;

#[hi::marker]
trait Array {}

#[hi::marker]
trait Uniform {}

#[hi::marker]
trait V {}

#[hi::mark(Array, Uniform, V)]
pub struct EmptyStruct;

#[hi::mark(Array, Uniform, V)]
pub enum EmptyEnum {}

pub mod type_ {
    use super::*;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestSingle<T>(PhantomData<T>);

    #[hi::mark(Array, Uniform, V)]
    pub struct TestMany<T, B, C>(PhantomData<(T, B, C)>);

    #[hi::mark(Array, Uniform, V)]
    pub struct TestWhereBoundSingle<T>(PhantomData<T>)
    where
        T: Default;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestWhereBoundMany<T, B>(PhantomData<(T, B)>)
    where
        T: Default,
        B: core::fmt::Debug;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestInnerBoundSingle<T: Default>(PhantomData<T>);

    #[hi::mark(Array, Uniform, V)]
    pub struct TestInnerBoundMany<T: Default + core::fmt::Debug, B: ?Sized>(PhantomData<(T, B)>);

    #[hi::mark(Array, Uniform, V)]
    pub struct TestMixedBoundMany<T: Default + core::fmt::Debug, B>(PhantomData<(T, B)>)
    where
        B: ?Sized;
}

pub mod const_ {
    use super::*;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestSingle<const N: usize>;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestMany<const N: usize, const M: usize>;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestSingleBound<const N: usize>
    where
        [i32; N]: Default;

    #[hi::mark(Array, Uniform, V)]
    pub struct TestManyBound<const N: usize, const M: usize>
    where
        [i32; N]: Default,
        [(); N]: Clone;
}

#[hi::mark(Array, Uniform, V)]
pub struct TestAll<T: Default, const N: usize>(PhantomData<T>)
where
    [T; N]: Sized;
