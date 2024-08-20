
#[macro_export]
macro_rules! denmark {
    ($ty:ty as $($traits:path),+ $(,)?) => {
        $(impl $traits for $ty { })+
    };
    ($ty:path as $($traits:path),+ $(,)?) => {
        $(impl $traits for $ty { })+
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
