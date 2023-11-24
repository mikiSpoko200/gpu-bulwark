use std::marker::PhantomData;
use super::prelude::*;
use crate::error;

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

pub struct Handle<R: Resource> {
    // this is needed to take R by value in drop which itself takes receiver by &mut self.
    resource: Option<R>,
}

impl<R> Handle<R>
where
    R: Resource
{
    pub fn new(resource: R) -> Self {
        Self { resource: Some(resource) }
    }
}

impl<R> Default for Handle<R>
where
    R: Resource + From<Object<R>>,
{
    fn default() -> Self {
        todo!();
    }
}

impl<R> Drop for Handle<R>
where
    R: Resource,
{
    fn drop(&mut self) {
        todo!();
    }
}

impl<R> std::ops::Deref for Handle<R>
where
    R: Resource,
{
    type Target = R;

    fn deref(&self) -> &Self::Target {
        self.resource
            .as_ref()
            .expect("resource maybe None only in Drop")
    }
}

impl<R> std::ops::DerefMut for Handle<R>
where
    R: Resource,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
            .as_mut()
            .expect("resource maybe None only in Drop")
    }
}

/// Handle to multiple homogeneous Resources.
pub struct MHandle<R: Resource> {
    resources: Vec<R>,
}

pub(crate) trait Bindable: Sized {
    fn bind(&self);
    fn unbind(&self);
}

pub trait Resource: Sized {
    type Ok;

    fn initialize(names: &mut [Name]) -> error::Result<Self::Ok>;

    fn free(names: &[Name]) -> error::Result<Self::Ok>;
}
