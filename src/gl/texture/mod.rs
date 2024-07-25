pub mod target;

use crate::gl::{self, texture};

use crate::prelude::internal::*;
use super::object::Allocator;
pub use target::Target;

pub struct TextureAllocator<T>(PhantomData<T>);

unsafe impl<T> Allocator for TextureAllocator<T>
where
    T: Target,
{
    fn allocate(names: &mut [u32]) {
        gl::call! {
            [panic]
            unsafe {
                glb::CreateTextures(T::VALUE as _, names.len() as _, names.as_mut_ptr())
            }
        }
    }

    fn free(names: &[u32]) {
        gl::call! {
            [panic]
            unsafe {
                glb::DeleteTextures(names.len() as _, names.as_ptr())
            }
        }
    }
}

pub struct Texture;
