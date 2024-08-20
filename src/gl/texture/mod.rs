pub mod target;
pub mod storage;
pub mod valid;
pub mod pixel;
pub mod image;

use crate::gl;
use crate::hlist::indexed;
use gl::texture;
use gl::target::Target as _;
use gl::object;
use gl::buffer;

use crate::prelude::internal::*;
use crate::gl::object::*;
pub use target::{Target, Buffer};
pub use storage::{Immutable, Mutable, Storage};


#[hi::mark(PartialObject, Object)]
pub struct TextureObject<T>(PhantomData<T>) where T: Target;

impl<T> Binder for TextureObject<T>
where T: Target
{
    fn bind(name: u32) {
        gl::call! {
            [panic]
            unsafe {
                glb::BindTexture(T::ID as _, name);
            }
        }
    }
}

unsafe impl<T> Allocator for TextureObject<T>
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

#[derive(dm::Deref)]
pub struct TextureState<T, K, InterFormat>
where
    T: texture::Target,
    K: storage::marker::Kind<Target = T>,
    InterFormat: InternalFormat,
{
    target: PhantomData<T>,
    #[deref]
    storage: storage::Storage<T, K, InterFormat, false>,
}

impl<T, K, InterFormat> TextureState<T, K, InterFormat>
where
    T: texture::Target,
    K: storage::marker::Kind<Target = T>,
    InterFormat: InternalFormat,
{
    const fn new(storage: Storage::<T, K, InterFormat, false>) -> Self {
        Self {
            storage,
            target: PhantomData,
        }
    }
}

pub trait InternalFormat {
    const ID: usize;
}

pub trait Format {
    const ID: usize;
}

#[derive(dm::Deref)]
pub struct Texture<T, K, InterFormat>
where
    T: Target,
    K: storage::marker::Kind<Target = T>,
    InterFormat: InternalFormat,
{
    #[deref]
    object: ObjectBase<TextureObject<T>>,
    state: TextureState<T, K, InterFormat>
}

impl<T, InterFormat> Texture<T, storage::Immutable<T>, InterFormat>
where
    T: Target + storage::marker::Internal + storage::AllocatorDispatch,
    InterFormat: InternalFormat,
{
    const TARGET: u32 = T::ID as _;
    const INTERNAL_FORMAT: u32 = InterFormat::ID as _;
}
