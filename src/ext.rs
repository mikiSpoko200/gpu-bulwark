
pub trait Array {
    const SIZE: usize;
    type Type;
}

impl<T, const N: usize> Array for [T; N] where T: Array {
    const SIZE: usize = N * T::SIZE;
    type Type = T;
}

impl Array for f32 {
    const SIZE: usize = 1;
    type Type = Self;
}

impl Array for f64 {
    const SIZE: usize = 1;
    type Type = Self;
}

impl Array for i32 {
    const SIZE: usize = 1;
    type Type = Self;
}

impl Array for u32 {
    const SIZE: usize = 1;
    type Type = Self;
}