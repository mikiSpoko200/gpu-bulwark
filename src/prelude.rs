use gl::types::GLenum;

pub trait Const<T> {
    const VALUE: T;
}

pub trait TypeEnum: Const<GLenum> { }

/// Implement trait that **just** extends `Const<T>`
///
/// It expects the following pattern:
/// `Trait for Type, Value, Const-Type`
#[macro_export]
#[allow(unused)]
macro_rules! impl_const_super_trait {
    ($super_trait:ident for $ty:ty, $value:expr) => {
        impl Const<crate::gl::types::GLenum> for $ty {
            const VALUE: crate::gl::types::GLenum = $value;
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
            let errors = crate::error::Error::poll_queue();
            if errors.len() > 0 {
                let message = errors.into_iter().map(ToString::to_string).collect::<Vec<_>>().join("\n");
                panic!("{message}");
            }
        }
    };
    (#[propagate] $invocation:stmt) => {
        $invocation
        let errors = crate::error::Error::poll_queue();
        if errors.len() > 0 { Err(errors) } else { Ok(()) }
    };
}

pub(crate) mod private {
    pub trait Sealed { }
}