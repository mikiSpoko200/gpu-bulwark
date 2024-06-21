use crate::mode;

pub mod constraint;

#[hi::marker]
pub trait Fulfiled {}

#[hi::marker]
pub trait Constraint<T> {}

/// Constraint for trait bound level validation.
#[hi::marker]
pub trait Valid<M> where M: mode::Validation { }

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