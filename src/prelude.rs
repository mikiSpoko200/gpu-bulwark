pub use crate::glsl;

/// Internal part of prelude;
pub(crate) mod internal {
    pub(crate) use crate::impl_target;
    pub(crate) use crate::utils::Disjoint;

    pub(crate) use std::marker::PhantomData;

    /// Wrapper for integer values that moves them into type system.
    /// Same trick is used in std here `https://doc.rust-lang.org/std/simd/prelude/struct.Simd.html`
    pub(crate) enum Const<const NUMBER: usize> { }
}

/// Implement trait that **just** extends `Const<T>`
///
/// It expects the following pattern:
/// `Trait for Type, Value, Const-Type`
#[macro_export]
#[allow(unused)]
macro_rules! impl_const_super_trait {
    ($super_trait:ident for $ty:ty, $value:expr) => {
        impl Const<glb::types::GLenum> for $ty {
            const VALUE: glb::types::GLenum = $value;
        }
        impl $super_trait for $ty {}
    };
    ($super_trait:ident for $ty:ty, $value:expr, $const_type:path) => {
        impl Const<$const_type> for $ty {
            const VALUE: $const_type = $value;
        }
        impl $super_trait for $ty {}
    };
}

