
pub trait Const<T> {
    const VALUE: T;
}

pub trait TypeEnum: Const<u32> {}

pub trait Disjoint {
    type Discriminant;
}
