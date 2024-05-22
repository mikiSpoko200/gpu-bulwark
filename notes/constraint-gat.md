

https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=3ab96b3526b6fa0984026734881ae952

```rust
use std::marker::PhantomData;


pub trait Fulfiled { }

pub trait Constraint {
    type Constrainer<T>: Fulfiled;
}

pub mod marker {
    pub trait Uniform { }
    
    impl Uniform for f32 { }
    impl Uniform for i32 { }
}

// Facade types that allow for picking constraints
pub mod constraint {
    pub struct Vector;
    pub struct Uniform;
}

// Types here implement bounds for generic parameters
mod constrainer {
    use std::marker::PhantomData;
    use super::Fulfiled;
    use super::marker;

    pub struct Vector<T>(PhantomData<T>);
    pub struct Uniform<T>(PhantomData<T>) where T: marker::Uniform;
    
    impl<T> Fulfiled for Vector<T> { }
    impl<T> Fulfiled for Uniform<T> where T: marker::Uniform { }
}

impl Constraint for constraint::Vector {
    type Constrainer<T> = constrainer::Vector<T>;
}

impl Constraint for constraint::Uniform {
    type Constrainer<T: marker::Uniform> = constrainer::Uniform<T>;
}

pub trait Buffer<C, T> where C: Constraint, C::Constrainer<T>: Fulfiled { }



pub struct VertexBuffer<T>(PhantomData<T>);

impl<T> Buffer<constraint::Vector, T> for VertexBuffer<T> { }

pub struct UniformBuffer<T>(PhantomData<T>);

impl<T> Buffer<constraint::Uniform, T> for UniformBuffer<T> where T: marker::Uniform { }
```