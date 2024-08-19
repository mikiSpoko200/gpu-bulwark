
https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=8cce536feb5561e3ea3f903e1e580e17

```
use std::marker::PhantomData;
std::simd::prelude::*;

pub struct Transparent;

pub unsafe trait FFI {
    type Layout;
}

pub trait Type: FFI {
    type Group;
}

pub trait Compatible<GLSL>: AsRef<<GLSL as FFI>::Layout>
where
    GLSL: Type<Group = Transparent>,
{ }

impl<T, GLSL> Compatible<GLSL> for T
where
    GLSL: Type<Group=Transparent>,
    T: AsRef<<GLSL as FFI>::Layout>
{ }

pub struct Vec<T, const SIZE: usize>(PhantomData<T>)
where
    T: Type;

impl<T, const N: usize> Type for Vec<T, N>
where
    T: Type,
{
    type Group = Transparent;
}

unsafe impl<T, const N: usize> FFI for Vec<T, N>
where
    T: Type,
{
    type Layout = [T; N];
}

pub type Vec2 = Vec<f32, 2>;
    
impl Type for f32 {
    type Group = Transparent;
}
unsafe impl FFI for f32 {
    type Layout = Self;
}

impl Type for f64 {
    type Group = Transparent;
}
unsafe impl FFI for f64 {
    type Layout = Self;
}

impl Type for i32 {
    type Group = ();
}
unsafe impl FFI for i32 {
    type Layout = Self;
}

fn test<G>(_: impl Compatible<G>) where G: Type<Group=Transparent> { }

fn main() {
    let foo = [1.0, 2.0f32];
    
    test::<Vec2>(&foo);
}
```