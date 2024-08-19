

### Emulating an Effect System with Rust's Type System

### Attempt 1 - partial application

[playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=9bf3105b0e65f1ed50a760eb00e803d3)

In Rust, the concept of an effect system isn't directly supported like in some functional languages (e.g., Haskell). However, you can emulate an effect system using Rust's type system, generics, and traits. The code you provided illustrates a way to enforce certain operations (or effects) at compile time through the use of traits, type parameters, and `PhantomData`. Here's a breakdown of how this can work:

```rust
use std::marker::PhantomData;

pub struct Buffer;

pub struct Texture;

pub struct Binder<T>(PhantomData<T>);

pub trait Effect { }

impl<T> Effect for Binder<T> { }

pub trait Bind: Sized {
    fn binder(&self) -> Binder<Self> {
        Binder(PhantomData)
    }
}

impl Bind for Buffer { }

pub struct VAO;

impl VAO {
    fn attrib_pointer(&mut self, buffer: Binder<Buffer>) -> impl FnOnce(Buffer) {
        let inner = move |buffer| {
            println!("using vao with guaranteed bound buffer");
        };
        inner
    }
}

fn main() {
    let mut vao = VAO;
    let buffer = Buffer;
    let binder = buffer.binder();
    vao.attrib_pointer(binder)(buffer);
}
```

#### 1. **PhantomData as a Marker**
   - `PhantomData` is a zero-sized marker type that carries information about a type without actually containing any value of that type.
   - In this code, `PhantomData<T>` is used to mark the `Binder<T>` type, where `T` represents the type of the resource being managed (e.g., `Buffer` or `Texture`).

#### 2. **Traits as Effect Interfaces**
   - The `Effect` trait represents an "effect" or an operation that can be performed. In a real effect system, this could represent actions like reading from a buffer, binding a texture, or any other operation that you want to enforce through the type system.
   - By implementing the `Effect` trait for `Binder<T>`, you're essentially stating that the binder can be used to perform operations on a resource of type `T`.

#### 3. **Binding Resources with a `Binder`**
   - The `Bind` trait provides a way to create a `Binder` for a given resource (in this case, `Buffer`). When you call `buffer.binder()`, you're creating a `Binder<Buffer>`, which can be passed around to indicate that a `Buffer` is bound and ready to be used.
   - This pattern can be extended to other resources (like `Texture`) by implementing `Bind` for them.

#### 4. **Enforcing Effects at Compile Time**
   - In the `VAO` struct, the `attrib_pointer` method takes a `Binder<Buffer>` as a parameter. This enforces that the method can only be called when a `Buffer` is bound.
   - The use of `impl FnOnce(Buffer)` as the return type allows the `Buffer` to be consumed in a specific operation (here, a simple print statement). The fact that `attrib_pointer` requires a `Binder<Buffer>` ensures that the `Buffer` is properly bound before this operation occurs.

#### 5. **Encapsulation and Safety**
   - This approach encapsulates the binding logic and ensures that operations requiring certain "effects" (like binding a buffer) are only available when the required conditions are met (e.g., the buffer is actually bound).
   - The Rust compiler checks that the types are used correctly, preventing runtime errors related to incorrect resource usage.

### Summary

By using `PhantomData`, traits, and type parameters, you can emulate an effect system in Rust where the type system ensures that certain operations are only performed when specific conditions are met. In this example:

- `Binder<T>` represents the effect of having a resource of type `T` bound.
- `Effect` is a marker trait that indicates a valid operation can be performed with the bound resource.
- The `attrib_pointer` method in `VAO` enforces that a `Buffer` must be bound before it can be used.

This pattern is powerful for ensuring safety and correctness in systems that involve complex resource management, such as graphics programming, where you need to ensure that resources are correctly bound before use.

### Attempt 2 - parameter passing

