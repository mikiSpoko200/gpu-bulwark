# Bounds on trait declarations vs trait implementations.

## The bad

First which comes to mind is that not placing bounds on type that's sole purpose is to implement one trait is nonsencial.
Otherwise type is trully and utterly useless and worst of all tools will not help you diagnose this problem quickly.

## The good

As I still have not learned my lesson, skipping bounds on type in vafour of boudns in blanket impl is that we can use generic trait.

```rust
// example is based on buffer target impl.

// Target 1
pub enum Array { }

// Target 1
pub enum Element { }

// ...

pub mod valid {
    use super::*;

    /// Trully generic validation bound.
    pub trait For<Target> { }

    // Now the question is:
    // will the compiler deduce that by consequence of orphan rule the blanket
    // below is **the only** possible implementation of For<Array> and as such `promote` bounds
    // so that `For<Array>` would imply bounds in blanket's `where` clause

    impl For<Array> for T: where ... { }
    impl For<Element> for T: where ... { }
}
```

## Answer

NO. LAME.

```rust
use std::marker::PhantomData;


pub trait Bound1 {
    const N: usize = 1;
}

pub trait Bound2 {
    const N: usize = 2;
}

pub trait Bound3 {
    const N: usize = 3;
}

pub enum Target1 { }

pub enum Target2 { }

pub enum Target3 { }

pub mod valid {
    use super::*;

    pub trait For<T> { }

    impl<T> For<Target1> for T where T: Bound1 { }
    impl<T> For<Target2> for T where T: Bound2 { }
    impl<T> For<Target3> for T where T: Bound3 { }
}

pub struct Buffer<T>(PhantomData<T>);

fn inner_needing_bound<T>() where T: Bound1 { }

impl Buffer<Target1> {
    fn data<D>(_: D) where D: valid::For<Target1> {
        inner_needing_bound::<D>();
    }
}
```

The above does not compile as of 1.79.

In he snipped above we see that despite the fact that compiler will enforce that there's only one impl block of `For<Target1>` due to it being a blanket impl it does not deduce that it can essentially treat boudns on `T` in said impl as super traits of `For<Target1>` and this is really quite sad since.

This proves that we cannot emulate super traits in any other way than by creating a alias and adding them explicitly. At the same time it demolishes any semblance of hope we had for `constraint` based api's due to lacking expressivness that alias traits do posses with regard to super traits.

Same applies to bounds on inner types in GATs.

## State of the art

For now it seems we're stuck with writing

```rust
pub mod valid {
    pub trait ForArray: /* HERE DEFINE THE CONTRACT USING `valid::` super traits */ { }
}
```

It does obviously work in the reverse direction -- if we require `T: Bound1` we can use it wherever `T: For<Target1>` is required.

This perhaps would allow us to separate trait bounds into two cathegories -- providers and consumers (check what rust uses internally). Validation traits would be consumers and bounds would be providers -- something like we currently have going on with `alias::TransparentType<Subtype=X>` and `valid::ForX` one of them provides requirements another one expects them -- does this make any sense? Make sure it's not just the matter of perspective and framing -- proove that there is some actual difference (idea: show that single Bound1 allows us to access multiple consumers?).

# To be discussed

At the moment it seems like validation / consumer traits are much shorter to name -- hey, but they are not aliases `alias` modules serve that purpose. They provide convinient way to provide requirements. Validation just consumes them.
