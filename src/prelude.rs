pub use crate::glsl;

/// Internal part of prelude;
pub(crate) mod internal {
    pub(crate) use std::marker::PhantomData;
    pub(crate) use sealed::sealed;

    /// Wrapper for integer values that moves them into type system.
    /// Same trick is used in std here `https://doc.rust-lang.org/std/simd/prelude/struct.Simd.html`
    pub(crate) struct Const<const NUMBER: usize>;
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

/// Wrapper for calling opengl functions.
///
/// In Debug mode it checks for errors and panics.
/// In Release it does nothing.
#[macro_export]
#[allow(unused)]
macro_rules! gl_call {
    (#[panic] $invocation:stmt) => {
        $invocation
        if cfg!(debug_assertions) {
            let errors = $crate::error::Error::poll_queue();
            if errors.len() > 0 {
                let message = errors.into_iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
                panic!("gl error: {message}");
            }
        }
    };
    (#[propagate] $invocation:stmt) => {
        $invocation
        let errors = $crate::error::Error::poll_queue();
        if errors.len() > 0 { Err(errors) } else { Ok(()) }
    };
}
