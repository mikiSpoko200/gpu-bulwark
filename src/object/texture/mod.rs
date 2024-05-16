use std::marker::PhantomData;

use crate::gl_call;

use super::resource::Allocator;

pub mod marker {
    use crate::prelude::Const;

    pub unsafe trait Target: Const<u32> { }
}

pub struct TextureAllocator<T>(PhantomData<T>) where;

unsafe impl<T> Allocator for TextureAllocator<T> where T: marker::Target {
    fn allocate(names: &mut [u32]) {
        gl_call! {
            #[panic]
            unsafe {
                gl::CreateTextures(T::VALUE as _, names.len() as _, names.as_mut_ptr())
            }
        }
    }

    fn free(names: &[u32]) {
        todo!()
    }
}

pub struct Texture;