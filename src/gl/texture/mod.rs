pub mod target;
pub mod storage;

use crate::gl;
use crate::hlist::indexed;
use gl::texture;
use gl::target::Target as _;
use gl::image;
use gl::object;
use gl::buffer;

use crate::prelude::internal::*;
use crate::gl::object::*;
pub use target::Target;
// pub use storage::Storage


impl<GL> storage::Storage for gl::Buffer<buffer::target::Texture, GL> { }

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

pub struct TextureState<T, K, InterFormat>
where
    T: texture::Target,
    K: storage::Kind,
    InterFormat: InternalFormat,
{
    target: PhantomData<T>,
    storage: PhantomData<K>,
    internal_format: PhantomData<InterFormat>,
    width: usize,
    height: usize,
}

impl<T, K, InterFormat> TextureState<T, K, InterFormat>
where
    T: texture::Target,
    K: storage::Kind,
    InterFormat: InternalFormat,
{
    /// Creates a new [`TextureState<S, InterFormat>`].
    const fn new(dimensions: [u32; ]) -> Self {
        Self {
            storage: PhantomData,
            internal_format: PhantomData,
            width,
            height,
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
    T: Target + storage::AllocatorDispatch<K>,
    K: storage::Kind,
    InterFormat: InternalFormat,
{
    #[deref]
    object: ObjectBase<TextureObject<T>>,
    state: TextureState<K, InterFormat>
}

impl<T, K, InterFormat> Texture<T, K, InterFormat>
where
    T: Target,
    K: storage::Kind,
    InterFormat: InternalFormat,
{
    const TARGET: u32 = T::ID as _;
    const INTERNAL_FORMAT: u32 = InterFormat::ID as _;

    fn create(width: usize, height: usize) -> Self {
        Self {
            object: Default::default(),
            state: TextureState::new(width),
        }
    }
}


impl<InterF> Texture<target::D2, storage::Immutable, InterF>
where 
    InterF: InternalFormat,
{
    /// Create new texture with immutable storage.
    fn create_with_storage_2d(width: usize, height: usize, levels: usize) -> Self { 
        let texture = Self::create(width, height);
        let _binder = texture.bind();

        gl::call! {
            [panic]
            unsafe {
                glb::TexStorage2D(
                    Self::TARGET,
                    levels as _,
                    Self::INTERNAL_FORMAT,
                    width as _,
                    height as _,
                );
            }
        }
    }

    fn sub_image_2d<F: Format>(&mut self, data: &[F], level: u32) {
        let _binder = self.bind();
        gl::call! {
            [panic]
            unsafe {
                glb::TexSubImage2D(
                    Self::TARGET,
                    level,
                    0, 0,
                    self.wi
                );
            }
        }
    }
}
