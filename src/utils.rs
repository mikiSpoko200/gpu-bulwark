use crate::hlist::{self, lhlist};


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

pub trait ConstIndexMut<I: hlist::counters::Index> {
    type Output<const N: usize>;

    fn const_index_mut<const INDEX: usize>(&self) -> Self::Output<INDEX>
    where
        Self: lhlist::Find<Self::Output<INDEX>, I>
    ;
}

pub trait ConstIndex<T> {
    type Output<const N: usize>;

    fn const_index<const INDEX: usize>(&self) -> Self::Output<INDEX>;
}

/// Maybe const eval allows some magic to happen with slices? then we could loosen the requirement for locations to be increasing.
/// FIXME: Besides increasing sequences would be painful to maintain if they were to be mutable - as in allow insertion and deletion different that pop / append
fn assert_const_indexable(lhs: usize, rhs: usize) {
    if !(lhs < rhs) { panic!("invalid types, lhs < rhs does not hold") }
}

pub trait ConstIndexHList<I> {

}
