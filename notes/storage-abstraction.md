

[playground perma-link](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=3f6f89d1d84f1a06e1eae5136e01e5d5)

```
use std::marker::PhantomData;

/// Abstrakcja sposobu przechowywania danych
pub trait Storage {
    type Store<T>;
}

/// Przechowywanie samego typu `T`, wynik ma 0 bajtów
pub struct Phantom;
impl Storage for Phantom {
    type Store<T> = PhantomData<T>;
}

/// Przechowywanie typu T jako T
pub struct Inline;
impl Storage for Inline {
    type Store<T> = T;
}

/// Przechowywanie na stercie
pub struct Boxed;
impl Storage for Boxed {
    type Store<T> = Box<T>;
}

pub struct Binding<T, Store>(Store::Store<T>) where Store: Storage;

impl<T> Binding<T, Phantom> {
    pub fn new(_: T) -> Self {
        Self(PhantomData)
    }
}

impl<T> Binding<T, Inline> {
    pub fn new(t: T) -> Self {
        Self(t)
    }
} 

impl<T> Binding<T, Boxed> {
    pub fn new(t: T) -> Self {
        Self(Box::from(t))
    }
} 


fn main() {
    let phantom = Binding::<_, Phantom>::new(1);
    let inline = Binding::<_, Inline>::new(1);
    let boxed = Binding::<_, Boxed>::new(1);
    
    println!("size of:");
    println!(" phantom: {}", std::mem::size_of_val(&phantom));
    println!("  inline: {}", std::mem::size_of_val(&inline));
    println!("   boxed: {}", std::mem::size_of_val(&boxed));
}
```