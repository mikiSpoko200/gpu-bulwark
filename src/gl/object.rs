
use crate::prelude::internal::*;
use crate::gl;

use gl::error;


mod private {
    
    pub(in crate::gl) mod access {
        pub trait Access {}

        #[hi::mark(Access)]
        pub enum Priv {}

        #[hi::mark(Access)]
        pub enum Pub {}
    }

    pub trait Bind: Sized {
        fn bind(name: u32);
        fn unbind() {
            Self::bind(0);
        }
    }
    pub unsafe trait Allocator: Sized {
        fn allocate(names: &mut [u32]);
    
        fn free(names: &[u32]);
    }
    pub trait PartialObject: Allocator { }
    
    pub trait Object: PartialObject + Bind { }
}
pub(in crate::gl) use private::*;

#[must_use]
pub struct BindGuard<B: Bind>(PhantomData<B>);

impl<B: Bind> BindGuard<B> {
    /// Create new binder.
    pub(in crate::gl) fn new(name: u32) -> Self {
        B::bind(name);
        Self(PhantomData)
    }
}

impl<B: Bind> Drop for BindGuard<B> {
    fn drop(&mut self) {
        B::unbind();
    }
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ObjectBase<O: PartialObject> {
    name: u32,
    object: PhantomData<O>,
}

impl<O: PartialObject> Default for ObjectBase<O> {
    fn default() -> Self {
        let mut name = 0;
        O::allocate(std::slice::from_mut(&mut name));
        Self {
            name,
            object: PhantomData,    
        }
    }
}

impl<O: PartialObject> Drop for ObjectBase<O> {
    fn drop(&mut self) {
        O::free(&[self.name]);
    }
}

impl<O: PartialObject> ObjectBase<O> {
    pub fn name(&self) -> u32 {
        self.name
    }
}

impl<O: Object> ObjectBase<O> {
    // Objects allow public binding of objects
    pub fn bind(&self) -> BindGuard<O> {
        BindGuard::<O>::new(self.name())
    }

    pub fn bound<T>(&self, f: impl FnOnce(&BindGuard<O>) -> T) -> T {
        let bind = self.bind();
        f(&bind)
    }
}