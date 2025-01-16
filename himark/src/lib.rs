
#[macro_export]
macro_rules! denmark {
    // Type -> impl many traits
    ($(type)? $ty:ty as $($traits:path),+ $(,)?) => {
        $(impl $traits for $ty { })+
    };
    ($(type)? $ty:path as $($traits:path),+ $(,)?) => {
        $(impl $traits for $ty { })+
    };
    
    // Trait -> impl for many types
    ($trait:path where $($ty:path),+ $(,)?) => {
        $(
            impl $trait for $ty { }
        )+
    };

    (impl <$type_var:ident : $bound:path> $ty:ty as $($traits:path),+ $(,)?) => {
        $(impl<$type_var: $bound> $traits for $ty { })+
    };
    (impl <$type_var:ident> $ty:ty as $($traits:path),+ $(,)?) => {
        $(impl<$type_var> $traits for $ty { })+
    };
}

#[cfg(feature = "attrs")]
extern crate himark_proc;

#[cfg(feature = "attrs")]
pub use himark_proc::{mark, marker};
