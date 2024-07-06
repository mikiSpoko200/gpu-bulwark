use std::marker::PhantomData;

use crate::gl;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Object<A>
where
    A: gl::object::Allocator
{
    name: u32,
    _allocator: PhantomData<A>,
}

impl<A: gl::object::Allocator> Object<A> {
    pub fn name(&self) -> u32 {
        self.name
    }
}

impl<A> Default for Object<A>
where
    A: gl::object::Allocator
{
    fn default() -> Self {
        let mut name = 0;
        A::allocate(std::slice::from_mut(&mut name));
        Self {
            name,
            _allocator: PhantomData,
        }
    }
}

impl<A> Drop for Object<A>
where
    A: gl::object::Allocator
{
    fn drop(&mut self) {
        A::free(&[self.name]);
    }
}
