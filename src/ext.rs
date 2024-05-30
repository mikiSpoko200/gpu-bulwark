use sealed::sealed;

#[sealed]
pub trait Array {
    const SIZE: usize;
    type Type;
}

#[sealed]
impl<T, const N: usize> Array for [T; N] {
    const SIZE: usize = N;
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
