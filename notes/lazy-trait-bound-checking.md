
# Lazy trait bound checking

This is super cool. associated consts can be produced by `const fn`s and value of these is not checked unless actually used.

This can be exceptionally useful for creating compile time checked 

```rust
use std::marker::PhantomData;

pub trait Foo {
    const BAR: usize;
}

pub struct One;
pub struct Two;

impl Foo for One {
    const BAR: usize = 1;
}
impl Foo for Two {
    const BAR: usize = 2;
}

pub struct None;
pub struct Some<T>(PhantomData<T>);

impl Foo for None {
    const BAR: usize = panic!("`None` does not implement Foo");
}

impl<T> Foo for Some<T> where T: Foo {
    const BAR: usize = T::BAR;
}

struct UseFoo<T: Foo>(PhantomData<T>);

fn dont_use_foo<T: Foo>() {
    let foo = UseFoo::<T>::new();
    println!("{}", foo.dont_use_assoc_const());
}

fn use_foo<T: Foo>() {
    let foo = UseFoo::<T>::new();
    println!("{}", foo.use_assoc_const());
}

impl<T: Foo> UseFoo<T> {
    pub fn new() -> Self {
        UseFoo::<T>(PhantomData)
    }

    pub fn dont_use_assoc_const(&self) -> String {
        String::from("siema")
    }
    
    pub fn use_assoc_const(&self) -> String {
        let _ = T::BAR;
        String::from("dangerous siema")
    }
}

fn main() {
    dont_use_foo::<Some<One>>();
    dont_use_foo::<Some<Two>>();
    dont_use_foo::<None>();
    
    use_foo::<Some<One>>();
    use_foo::<Some<Two>>();
    // use_foo::<None>();
}
```