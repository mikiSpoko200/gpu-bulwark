
pub trait Const<T> {
    const VALUE: T;
}

pub trait TypeEnum: Const<u32> {}
