pub use crate::glsl;
pub use crate::gl;

/// Internal part of prelude;
pub(crate) mod internal {
    pub(crate) use crate::impl_target;
    pub(crate) use crate::utils::Disjoint;

    pub(crate) use std::marker::PhantomData;

    /// Wrapper for integer values that moves them into type system.
    /// Same trick is used in std here `https://doc.rust-lang.org/std/simd/prelude/struct.Simd.html`
    pub enum Const<const NUMBER: usize> { }
}
