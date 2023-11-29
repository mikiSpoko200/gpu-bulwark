use super::prelude::*;
use crate::error;
use crate::target;
use std::marker::PhantomData;

// note: this must use another trait that allows for binding of arbitrary
// pub struct Binder<'obj, B>(&'obj Name, PhantomData<B>) where B: Bindable;
//
// impl<'obj, B> Binder<'obj, B> where B: Bindable {
//     pub fn bind(object: &'obj Name) -> Self {
//         Self(object, Default::default())
//     }
// }
//
// impl<'obj, B> Drop for Binder<'obj, B>
// where
//     B: Bindable
// {
//     fn drop(&mut self) {
//         B::
//     }
// }

pub(crate) trait Bindable: Sized {
    fn bind(&self);
    fn unbind(&self);
}

///
pub unsafe trait Allocator: Sized {
    fn allocate(names: &mut [u32]);

    fn free(names: &[u32]);
}
