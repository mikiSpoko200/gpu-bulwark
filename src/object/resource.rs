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
    R: Resource + From<Object>,
{
    fn default() -> Self {
        Self {
            resource: Some(manager::create()),
        }
    }
}

impl<R> Drop for Handle<R>
where
    R: Resource,
{
    fn drop(&mut self) {
        // unwrap does not panic since single value drop is well defined.
        manager::delete(
            // the only place that we move resource out of option is here in `Drop` so unwrap is ok.
            self.resource.take().unwrap(),
        )
        .unwrap();
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

/// Adapters that encapsulate Resource lifetime management.
pub(crate) mod manager {
    use crate::error;
    use crate::object::prelude::{Name, Object};
    use crate::object::resource::Resource;

    pub fn create<R>() -> R
    where
        R: Resource + From<Object>,
    {
        let mut name = [Default::default()];
        R::initialize(&mut name).expect("glCreate functions do not error when n >= 0");
        R::from(Object(name[0]))
    }

    pub fn delete<R>(r: R) -> error::Result<R::Ok>
    where
        R: Resource,
    {
        let name: Name = r.into().0;
        R::free(&[name])
    }

    // unsafe: N mustn't be usize since there cannot be that many gl objects
    pub fn static_bulk_delete<R, const N: usize>(resources: [R; N]) -> error::Result<R::Ok>
    where
        R: Resource,
    {
        let names = resources.map(|r| r.into().0);
        R::free(&names)
    }

    pub fn dyn_bulk_delete<I, R>(resources: I) -> error::Result<R::Ok>
    where
        I: Iterator<Item = R>,
        R: Resource,
    {
        let names: Vec<Name> = resources.map(|r| r.into().0).collect();
        R::free(&names)
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

pub trait Resource: Sized + Into<Object> {
    type Ok;

    fn initialize(names: &mut [Name]) -> error::Result<Self::Ok>;

    fn free(names: &[Name]) -> error::Result<Self::Ok>;
}
