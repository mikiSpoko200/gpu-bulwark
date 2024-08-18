
use crate::gl::buffer::target;
use crate::prelude::internal::*;
use crate::{ext, gl};
use gl::texture::storage;

use super::valid;

/// NOTE: Target dispatches storage allocation (https://www.khronos.org/opengl/wiki/Texture_Storage#Anatomy_of_storage)

pub mod _valid {
    pub trait ForMultiSample { }
}

/// Dimensionality of storage using given target
pub trait Dimensionality {
    type Dimensions: ext::Array;
}

macro_rules! impl_dimensionality {
    ([$dim:literal] $($target:ty),+ $(,)?) => {
        $(
            impl Dimensionality for $target { type Dimensions = [usize; $dim]; }
        )+
    };
}

impl_dimensionality! { [1] D1, Buffer }
impl_dimensionality! { [2] D2, Rectangle, D1Array, CubeMap, D2MultiSample }
impl_dimensionality! { [3] D3, CubeMapArray, D2Array, D2MultiSampleArray }

#[hi::marker]
pub trait Target: gl::target::Target + Dimensionality { }


/// NOTE: We cannot write blankets over `const N` since targets cannot be assigned in generic fashion - they need to be listed out.
/// NOTE: Due to this `D<N>` does not impl `target::Target` - only specific types do like `D<1>`.
/// NOTE: Thats why we need to provide extra bound in blanket impl for `Target` for `Self` to be `texture::Target`.
/// NOTE: note however that we don't need to add such bound on all discretely implemented traits - see that Dimensionality implemented via blanket is not necessary.
/// FIXME: Fact that Target is not implemented explicitly for `D<N>` disallows its direct usage in types like `Array<D<N>>` since D<N> is not `Target`. 
/// NOTE: It can still be used in context where `Target` is abstract.
/// NOTE: This bound prohibits us from using `hi::mark` with `Target` since `target::Target` is not implemented for all `D<N>`.
pub enum D<const N: usize> { }

pub type D1 = D<1>;
pub type D2 = D<2>;
pub type D3 = D<3>;

hi::denmark! { D1 as Target, storage::marker::Internal }
hi::denmark! { D2 as Target, storage::marker::Internal }
hi::denmark! { D3 as Target, storage::marker::Internal }

// TODO: further constrain `T` here to be `D1` or `D1`.
pub struct Array<T>(PhantomData<T>) where T: Target;

pub type D1Array = Array<D1>;
pub type D2Array = Array<D2>;

hi::denmark! { D1Array as Target, storage::marker::Internal }
hi::denmark! { D2Array as Target, storage::marker::Internal }

#[hi::mark(Target, storage::marker::Internal)]
pub enum Rectangle { }

#[hi::mark(Target)]
pub enum Buffer { }

#[hi::mark(Target, storage::marker::Internal)]
pub enum CubeMap { }

pub type CubeMapArray = Array<CubeMap>;

hi::denmark! { CubeMapArray as Target, storage::marker::Internal }

pub struct MultiSample<T>(PhantomData<T>) where T: Target;

pub type D2MultiSample = MultiSample<D2>;
pub type D2MultiSampleArray = Array<MultiSample<D2>>;

hi::denmark! { D2MultiSample as Target, storage::marker::Internal }
hi::denmark! { D2MultiSampleArray as Target, storage::marker::Internal }

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
