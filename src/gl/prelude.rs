use std::marker::PhantomData;

pub use glb::types::GLuint as Name;
use crate::gl;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Object<A>
where
    A: gl::resource::Allocator
{
    name: u32,
    _allocator: PhantomData<A>,
}

impl<A: gl::resource::Allocator> Object<A> {
    pub fn name(&self) -> u32 {
        self.name
    }
}

impl<A> Default for Object<A>
where
    A: gl::resource::Allocator
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
    A: gl::resource::Allocator
{
    fn drop(&mut self) {
        A::free(&[self.name]);
    }
}
