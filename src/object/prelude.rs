use std::marker::PhantomData;

pub use gl::types::GLuint as Name;

use super::resource::Resource;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct Object<R: Resource> {
    pub name: u32,
    pub outer_resource: PhantomData<R>,
}

impl<R: Resource> Object<R> {
    pub fn new(name: u32) -> Self { 
        Self {
            name,
            ..Default::default()
        }
    }
}

impl<R: Resource> Default for Object<R> {
    fn default() -> Self {
        let mut name = 0;
        R::initialize(&mut [name]).unwrap();
        Self { 
            name,
            outer_resource: Default::default()
        }
    }
}

impl<R: Resource> Drop for Object<R> {
    fn drop(&mut self) {
        R::free(&[self.name]).unwrap();
    }
}
