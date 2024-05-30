use frunk::hlist::Selector;
use gl::types::GLenum;

pub trait ArrayExt {
    const LENGTH: usize;
    type T;
}

impl<T, const N: usize> ArrayExt for [T; N] {
    const LENGTH: usize = N;
    type T = T;
}

pub trait Storage {
    type Store<T>;
}

pub trait Const<T> {
    const VALUE: T;
}

pub trait TypeEnum: Const<GLenum> {}

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

#[macro_export]
macro_rules! impl_default_without_bounds {
    ($type:ty) => {
        impl std::default::Default for $type:ty {
            fn default() -> Self {
                Self { .. Default::default() }
            }
        }
    };
}

pub(crate) mod private {
    pub trait Sealed {}
}

pub trait HList: Sized {
    const LENGTH: usize;
    const INDEX: usize;

    #[inline]
    fn len(&self) -> usize {
        Self::LENGTH
    }

    fn append<T>(self, elem: T) -> (Self, T) {
        (self, elem)
    }
}

impl HList for () {
    const INDEX: usize = 1;
    const LENGTH: usize = 0;
}

impl<H: HList, T> HList for (H, T) {
    const INDEX: usize = H::INDEX + 1;
    const LENGTH: usize = H::LENGTH + 1;
}

pub trait HListExt: HList {
    type Head: HList;
    type Tail;

    fn get<T, Index>(&self) -> &T
    where
        Self: Selector<T, Index>,
    {
        Selector::get(self)
    }

    fn pop(self) -> (Self::Head, Self::Tail);
}

impl<H: HList, T> HListExt for (H, T) {
    type Head = H;

    type Tail = T;

    fn pop(self) -> (Self::Head, Self::Tail) {
        self
    }
}
