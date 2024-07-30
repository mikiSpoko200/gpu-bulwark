pub mod target;
pub mod storage;

use crate::gl;
use gl::texture;
use gl::target::Target as _;
use gl::image;

use crate::prelude::internal::*;
use super::object::Allocator;
pub use target::Target;

pub use storage::marker::Storage;

pub struct TextureAllocator<T>(PhantomData<T>);

unsafe impl<T> Allocator for TextureAllocator<T>
where
    T: Target,
{
    fn allocate(names: &mut [u32]) {
        gl::call! {
            [panic]
            unsafe {
                glb::CreateTextures(T::ID as _, names.len() as _, names.as_mut_ptr())
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

pub struct Texture<T, S, Format>
where
    T: Target,
    S: Storage,
{
    target: PhantomData<T>,
    storage: S,
    format: Format,
}

impl<F> Texture<target::D1, storage::Immutable, F> {
    // fn storage_1d(width: usize, levels: usize) -> gl::Result<Self> { 
    //     let result = gl::call! {
    //         [propagate]
    //         unsafe {
    //             glb::TexStorage1D(
    //                 target::D1::ID as _,
    //                 levels as _,
                    
    //             )
    //         }
    //     }?
    // }
}

