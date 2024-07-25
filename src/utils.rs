
pub trait Const<T> {
    const VALUE: T;
}

pub trait TypeEnum: Const<u32> { }

pub trait Disjoint {
    type Discriminant;
}

#[macro_export]
macro_rules! disjoint {
    ($([$discriminant:path] $($ty:path)+,);+ $(;)?) => {
        $(
            impl $crate::utils::Disjoint for $ty {
                type Discriminant = $discriminant;
            }
        )+
    };
    ($([$discriminant:path] $($ty:ty),+);+ $(;)?) => {
        $(
            $(
                impl $crate::utils::Disjoint for $ty {
                    type Discriminant = $discriminant;
                }
            )+
        )+
    };
}