//! Abstractions for type level validation.


/// Abstraction of compile-time validation using `const fn`.
pub trait ConstFnValid {
    const VALID: ();
}

pub trait ValidExt: ConstFnValid {
    fn validated(self) -> Self;
}

impl<T> ValidExt for T where T: ConstFnValid {
    fn validated(self) -> Self {
        let _: () = Self::VALID;
        self
    }
}