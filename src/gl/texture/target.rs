
use crate::prelude::internal::*;
use crate::gl;
use gl::texture::storage;

/// NOTE: Target dispatches storage allocation (https://www.khronos.org/opengl/wiki/Texture_Storage#Anatomy_of_storage)

pub mod _valid {
    pub trait ForMultiSample { }
}

#[hi::marker]
pub trait Target: gl::target::Target + Dimensionality { }

pub enum D<const N: usize> { }

pub type D1 = D<1>;
pub type D2 = D<2>;
pub type D3 = D<3>;

hi::denmark! { D1 as Target }
hi::denmark! { D2 as Target }
hi::denmark! { D3 as Target }

pub struct Array<T>(PhantomData<T>) where T: Target;

pub type D1Array = Array<D1>;
pub type D2Array = Array<D2>;

hi::denmark! { D1Array as Target }
hi::denmark! { D2Array as Target }

#[hi::mark(Target)]
pub enum Rectangle { }

#[hi::mark(Target)]
pub enum Buffer { }

#[hi::mark(Target)]
pub enum CubeMap { }

pub type CubeMapArray = Array<CubeMap>;

hi::denmark! { CubeMapArray as Target }

pub struct MultiSample<T>(PhantomData<T>) where T: Target;

pub type D2MultiSample = MultiSample<D2>;
pub type D2MultiSampleArray = Array<MultiSample<D2>>;

hi::denmark! { D2MultiSample as Target }
hi::denmark! { D2MultiSampleArray as Target }

impl_target! { D1 as TEXTURE_1D }
impl_target! { D2 as TEXTURE_2D }
impl_target! { D3 as TEXTURE_3D }
impl_target! { D1Array as TEXTURE_1D_ARRAY }
impl_target! { D2Array as TEXTURE_2D_ARRAY }
impl_target! { Rectangle as TEXTURE_RECTANGLE }
impl_target! { Buffer as TEXTURE_BUFFER }
impl_target! { CubeMap as TEXTURE_CUBE_MAP }
impl_target! { CubeMapArray as TEXTURE_CUBE_MAP_ARRAY }
impl_target! { D2MultiSample as TEXTURE_2D_MULTISAMPLE }
impl_target! { D2MultiSampleArray as TEXTURE_2D_MULTISAMPLE_ARRAY }


/// Dimensionality of storage using given target
pub trait Dimensionality {
    type Index;
}


macro_rules! impl_dimensionality {
    ([$dim:literal] $($target:ty),+ $(,)?) => {
        $(
            impl Dimensionality for $target { type Index = [u32; $dim]; }
        )+
    };
}

impl_dimensionality! { [1] D1, Buffer }
impl_dimensionality! { [2] D2, Rectangle, D1Array, CubeMap, D2MultiSample }
impl_dimensionality! { [3] D3, CubeMapArray, D2Array, D2MultiSampleArray }
