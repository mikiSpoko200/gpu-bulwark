use std::marker::PhantomData;

use sealed::sealed;

#[sealed]
pub trait Array {
    const SIZE: usize;
    type Type;
}

pub struct ArrayLayout<T, const SIZE: usize>(PhantomData<T>);

#[sealed]
impl<T, const N: usize> Array for [T; N] where T: Array {
    const SIZE: usize = N * T::SIZE;
    type Type = T;
}

#[sealed]
impl Array for f32 {
    const SIZE: usize = 1;
    type Type = Self;
}

#[sealed]
impl Array for f64 {
    const SIZE: usize = 1;
    type Type = Self;
}

#[sealed]
impl Array for i32 {
    const SIZE: usize = 1;
    type Type = Self;
}

#[sealed]
impl Array for u32 {
    const SIZE: usize = 1;
    type Type = Self;
}
