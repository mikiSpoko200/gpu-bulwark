use std::marker::PhantomData;

pub use gl::types::GLuint as Name;

use super::resource::Allocator;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Object<A: Allocator> {
    name: u32,
    _allocator: PhantomData<A>,
}

impl<A: Allocator> Object<A> {
    pub fn name(&self) -> u32 {
        self.name
    }
}

impl<A: Allocator> Default for Object<A> {
    fn default() -> Self {
        let mut name = 0;
        A::allocate(&mut [name]);
        Self {
            name,
            _allocator: PhantomData,
        }
    }
}

impl<A: Allocator> Drop for Object<A> {
    fn drop(&mut self) {
        A::free(&[self.name]);
    }
}
