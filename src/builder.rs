

pub(crate) mod private {
    pub trait Sealed { }
}

pub struct Completed<T>(pub T);

pub trait Builder: private::Sealed {
    type Output;

    fn build(self) -> Completed<Self::Output>;
}