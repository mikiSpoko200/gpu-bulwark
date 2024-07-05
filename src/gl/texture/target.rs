
use crate::prelude::internal::*;
use crate::gl;
use crate::impl_target;

pub mod _valid {
    pub trait ForMultiSample { }
}

pub trait Target: gl::target::Target {
    const VALUE: usize;
}

pub enum D<const N: usize> { }

pub struct Array<T>(PhantomData<T>) where T: Target;

pub struct MultiSample<T>(PhantomData<T>);

pub enum Rectangle { }

pub enum BUffer { }

pub enum CubeMap { }


impl_target! { D<1> as TEXTURE_1D }
impl_target! { D<2> as TEXTURE_2D }
impl_target! { D<3> as TEXTURE_3D }
impl_target! { Array<D<1>> as TEXTURE_1D_ARRAY }
impl_target! { Array<D<2>> as TEXTURE_2D_ARRAY }
impl_target! { Rectangle as TEXTURE_RECTANGLE }
impl_target! { BUffer as TEXTURE_BUFFER }
impl_target! { CubeMap as TEXTURE_CUBE_MAP }
impl_target! { Array<CubeMap> as TEXTURE_CUBE_MAP_ARRAY }
impl_target! { MultiSample<D<2>> as TEXTURE_2D_MULTISAMPLE }
impl_target! { Array<MultiSample<D<2>>> as TEXTURE_2D_MULTISAMPLE_ARRAY }

pub type D1 = D<1>;
pub type D2 = D<2>;
pub type D3 = D<3>;
pub type D1Array = Array<D1>;
pub type D2Array = Array<D2>;
pub type CubeMapArray = Array<CubeMap>;
pub type D2MultiSample = MultiSample<D2>;
pub type D2MultiSampleArray = Array<MultiSample<D2>>;
