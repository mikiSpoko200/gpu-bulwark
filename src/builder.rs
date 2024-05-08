

pub(crate) mod private {
    pub trait Sealed { }
}

#[allow(unused)]
pub struct Completed<T>(pub T);

pub trait Builder: private::Sealed {
    type Output;

    fn build(self) -> Completed<Self::Output>;
}