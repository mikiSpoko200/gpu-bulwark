pub mod target;
pub mod storage;
pub mod valid;
pub mod pixel;
pub mod image;

use std::ops::Deref;

use crate::gl;
use crate::glsl;
use crate::ts;
use crate::hlist::indexed;
use gl::texture;
use gl::target::Target as _;
use gl::object;
use gl::buffer;
use pixel::channels::Channels;
use storage::marker::Kind;

use crate::prelude::internal::*;
use crate::gl::object::*;
pub use target::{Target, Buffer};
pub use storage::{Immutable, Mutable, Storage};

#[hi::mark(PartialObject, Object)]
pub struct TextureObject<T>(PhantomData<T>) where T: Target;

impl<T> Binder for TextureObject<T>
where
    T: Target
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
pub struct TextureState<T, K, F>
where
    T: texture::Target,
    K: storage::marker::Kind<Target = T>,
    F: image::marker::Format,
{
    target: PhantomData<T>,
    #[deref]
    storage: storage::Storage<T, K, F, false>,
}

impl<T, K, F> TextureState<T, K, F>
where
    T: texture::Target,
    K: storage::marker::Kind<Target = T>,
    F: image::marker::Format,
{
    const fn new(storage: Storage::<T, K, F, false>) -> Self {
        Self {
            storage,
            target: PhantomData,
        }
    }
}

pub trait MaybeFormat: ts::Maybe + texture::image::marker::Format { }

hi::denmark! { ts::None as MaybeFormat }
impl texture::image::marker::Format for ts::None {
    const ID: u32 = panic!("not implemented");
    type BaseFormat = texture::image::format::RGBA;
    type Output = i32;
    
    type Composition = texture::image::marker::Aggregate;
    type ComponentType = u8;
    // type Kind = gl::types::Float;
}

impl<F> MaybeFormat for ts::Some<F> where F: texture::image::marker::Format { }
impl<F> texture::image::marker::Format for ts::Some<F> where F: texture::image::marker::Format {
    const ID: u32 = F::ID;
    type BaseFormat = F::BaseFormat;
    type Output = F::Output;
    
    type Composition = F::Composition;
    type ComponentType = F::ComponentType;
}

pub struct Builder<T, K, F>
where
    T: texture::Target,
    K: storage::marker::Kind<Target = T>,
    F: MaybeFormat,
{
    object: ObjectBase<TextureObject<T>>,
    kind: PhantomData<(K, F)>,
}

impl<T, K> Builder<T, K, ts::None>
where
    T: texture::Target,
    K: storage::marker::Kind<Target = T>,
{
    fn new() -> Self {
        Self {
            object: ObjectBase::<TextureObject<T>>::default(),
            kind: PhantomData,
        }
    }

    fn internal_format<F: MaybeFormat>(self) -> Builder<T, K, F> {
        Builder {
            object: self.object,
            kind: PhantomData,
        }
    }
}

#[derive(dm::Deref)]
pub struct Texture<T, K, InterFormat>
where
    T: Target,
    K: storage::marker::Kind<Target = T>,
    InterFormat: image::marker::Format,
{
    #[deref]
    object: ObjectBase<TextureObject<T>>,
    state: TextureState<T, K, InterFormat>
}


use image::marker::BaseFormat;


impl<D1Target, Kind, InternalFormat> Texture<D1Target, Kind, InternalFormat>
where
    D1Target: texture::Target<Dimensions = [usize; 1]>,
    Kind: storage::marker::Storage<Target=D1Target, Signature = storage::signature::Storage1D>,
    InternalFormat: image::marker::Format,
{
    pub fn create_with_storage_1d(width: usize) -> Self {
        let mut object = ObjectBase::default();
        let binder = object.bind();

        let storage = Storage::storage_1d(&binder, width);
        Self { object, state: TextureState::new(storage) }
    }

    pub fn sub_image_1d<Channels: pixel::channels::Channels>(
        &mut self, 
        x_range: impl std::ops::RangeBounds<usize>, 
        pixels: &[impl pixel::Pixel<Components = Channels::Components, Type = InternalFormat::ComponentType>]
    )
    where
        Channels: pixel::valid::ForImageBaseFormat<InternalFormat::BaseFormat>,
        (Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind): pixel::FormatToken,
    {
        let binder = self.bind();
        self.state.storage.sub_image_1d(&binder, x_range, pixels)
    }
}

impl<D2Target, Kind, InternalFormat> Texture<D2Target, Kind, InternalFormat>
where
    D2Target: texture::Target<Dimensions = [usize; 2]>,
    Kind: storage::marker::Storage<Target=D2Target, Signature = storage::signature::Storage2D>,
    InternalFormat: image::marker::Format,
{
    pub fn create_with_storage_2d(width: usize, height: usize) -> Self {
        let mut object = ObjectBase::default();
        let binder = object.bind();

        let storage = Storage::storage_2d(&binder, width, height);
        Self {
            object,
            state: TextureState::new(storage),
        }
    }

    pub fn sub_image_2d<
        Channels: pixel::channels::Channels,
        Pixel: pixel::Pixel<Components = Channels::Components, Type = InternalFormat::ComponentType>,
    >(
        &mut self,
        x_range: impl std::ops::RangeBounds<usize>, 
        y_range: impl std::ops::RangeBounds<usize>,
        pixels: &[Pixel]
    )
    where
        Channels: pixel::valid::ForImageBaseFormat<InternalFormat::BaseFormat>,
        (Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind): pixel::FormatToken,
    {
        let binder = self.bind();
        self.state.storage.sub_image_2d(&binder, x_range, y_range, pixels);
    }
}

impl<D3Target, Kind, InternalFormat> Texture<D3Target, Kind, InternalFormat>
where
    D3Target: texture::Target<Dimensions = [usize; 3]>,
    Kind: storage::marker::Storage<Target=D3Target, Signature = storage::signature::Storage3D>,
    InternalFormat: image::marker::Format,
{
    pub fn create_with_storage_3d(_: &object::Bind<TextureObject<D3Target>>, width: usize, height: usize, depth: usize) -> Self {
        let mut object = ObjectBase::default();
        let binder = object.bind();

        let storage = Storage::storage_3d(&binder, width, height, depth);

        Self {
            object,
            state: TextureState::new(storage),
        }
    }
    pub fn sub_image_3d<
        Channels: pixel::channels::Channels,
        Pixel: pixel::Pixel<Components = Channels::Components, Type = InternalFormat::ComponentType>,
    >(
        &mut self, 
        x_range: impl std::ops::RangeBounds<usize>,
        y_range: impl std::ops::RangeBounds<usize>,
        z_range: impl std::ops::RangeBounds<usize>,
        pixels: &[Pixel]
    )
    where
        Channels: pixel::valid::ForImageBaseFormat<InternalFormat::BaseFormat>,
        (Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind): pixel::FormatToken,
    {
        let binder = self.bind();
        self.state.storage.sub_image_3d(&binder, x_range, y_range, z_range, pixels);
    }
}

#[derive(dm::Deref, dm::DerefMut)]
pub struct TextureUnit<Target, Kind, InternalFormat, const INDEX: usize>(Texture<Target, Kind, InternalFormat>)
where
    Target: target::Target, 
    Kind: storage::marker::Kind<Target = Target>, 
    InternalFormat: texture::image::marker::Format,
;

impl<Target, Kind, InternalFormat, const INDEX: usize> TextureUnit<Target, Kind, InternalFormat, INDEX>
where
    Target: target::Target, 
    Kind: storage::marker::Kind<Target = Target>, 
    InternalFormat: texture::image::marker::Format,
{
    pub fn new<const N: usize>(texture: Texture<Target, Kind, InternalFormat>) -> gl::Result<TextureUnit<Target, Kind, InternalFormat, N>> {
        let binder = texture.bind();
        gl::call! {
            [propagate]
            unsafe {
                glb::BindTextureUnit(N as _, texture.name());
            }
        }.map(|_| TextureUnit(texture))
    }
}

impl<Target, Kind, InternalFormat, const INDEX: usize> gl::program::Resource for TextureUnit<Target, Kind, InternalFormat, INDEX>
where
    Target: target::Target, 
    Kind: storage::marker::Kind<Target = Target>, 
    InternalFormat: texture::image::marker::Format,
{
    type UniformVariable = glsl::GSampler<Target, InternalFormat::Output>;
    
    fn opaque_uniform_variable<const BINDING: usize>(&self) -> glsl::variable::OpaqueUniformVariable<Self::UniformVariable, BINDING> {
        glsl::variable::OpaqueUniformVariable::default()
    }

}

pub fn units() -> TextureUnits<()> {
    TextureUnits::<()>::default()
}

pub trait Binders {
    type Binders;

    fn binders(&self) -> Self::Binders;
}


pub struct TextureUnits<Handles>(Handles) where Handles: Binders;

impl Binders for () {
    type Binders = ();

    fn binders(&self) -> Self::Binders { () }
}

impl<H, Target, Kind, InternalFormat, const BINDING: usize> Binders for (H, &TextureUnit<Target, Kind, InternalFormat, BINDING>)
where
    H: Binders,
    Target: texture::Target,
    Kind: texture::storage::marker::Kind<Target = Target>,
    InternalFormat: texture::image::marker::Format,
{
    type Binders = (H::Binders, object::Bind<texture::TextureObject<Target>>);

    fn binders(&self) -> Self::Binders {
        (self.0.binders(), self.1.object.bind())
    }
}

impl Default for TextureUnits<()> {
    fn default() -> Self {
        Self(())
    }
}

impl<Handles> TextureUnits<Handles> where Handles: Binders {
    pub fn add<Target, Kind, InternalFormat, const BINDING: usize>(self, unit: &TextureUnit<Target, Kind, InternalFormat, BINDING>) -> TextureUnits<(Handles, &TextureUnit<Target, Kind, InternalFormat, BINDING>)> 
    where
        Target: texture::Target,
        Kind: texture::storage::marker::Kind<Target = Target>,
        InternalFormat: texture::image::marker::Format,
    {
        TextureUnits((self.0, unit))
    }

    pub fn binders(&self) -> Handles::Binders {
        self.0.binders()
    }
}
