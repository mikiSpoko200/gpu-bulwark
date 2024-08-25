use std::marker::PhantomData;

use crate::gl::texture;
use crate::gl;

pub trait Output { type Kind: gl::types::Kind; }

impl Output for f32 { type Kind = gl::types::Float;   }
impl Output for i32 { type Kind = gl::types::Integer; }
impl Output for u32 { type Kind = gl::types::Integer; }

pub struct Shadow<Target>(PhantomData<Target>) where Target: texture::Target;

pub struct GSampler<Target, O>(PhantomData<(Target, O)>)
where
    Target: texture::Target,
    O: Output
;

impl<Target, O> Default for GSampler<Target, O>
where
    Target: texture::Target,
    O: Output
{
    fn default() -> Self {
        Self(Default::default())
    }
}

type Sampler<Target> = GSampler<Target, f32>;

pub type Sampler1D                = Sampler<texture::target::D1>;
pub type Sampler1DShadow          = Sampler<Shadow<texture::target::D1>>;
pub type Sampler1DArray           = Sampler<texture::target::D1Array>;
pub type Sampler1DArrayShadow     = Sampler<Shadow<texture::target::D1Array>>;
pub type Sampler2D                = Sampler<texture::target::D2>;
pub type Sampler2DShadow          = Sampler<Shadow<texture::target::D2>>;
pub type Sampler2DArray           = Sampler<texture::target::D2Array>;
pub type Sampler2DArrayShadow     = Sampler<Shadow<texture::target::D2Array>>;
pub type Sampler3D                = Sampler<texture::target::D3>;
pub type Sampler2DMS              = Sampler<texture::target::D2MultiSample>;
pub type Sampler2DMSArray         = Sampler<texture::target::D2MultiSampleArray>;
pub type Sampler2DRect            = Sampler<texture::target::Rectangle>;
pub type Sampler2DRectShadow      = Sampler<Shadow<texture::target::Rectangle>>;
pub type Sampler2DCube            = Sampler<texture::target::CubeMap>;
pub type Sampler2DCubeShadow      = Sampler<Shadow<texture::target::CubeMap>>;
pub type Sampler2DCubeArray       = Sampler<texture::target::CubeMapArray>;
pub type Sampler2DCubeArrayShadow = Sampler<Shadow<texture::target::CubeMapArray>>;
pub type SamplerBuffer            = Sampler<texture::target::Buffer>;


type ISampler<Target> = GSampler<Target, i32>;

pub type ISampler1D                = ISampler<texture::target::D1>;
pub type ISampler1DShadow          = ISampler<Shadow<texture::target::D1>>;
pub type ISampler1DArray           = ISampler<texture::target::D1Array>;
pub type ISampler1DArrayShadow     = ISampler<Shadow<texture::target::D1Array>>;
pub type ISampler2D                = ISampler<texture::target::D2>;
pub type ISampler2DShadow          = ISampler<Shadow<texture::target::D2>>;
pub type ISampler2DArray           = ISampler<texture::target::D2Array>;
pub type ISampler2DArrayShadow     = ISampler<Shadow<texture::target::D2Array>>;
pub type ISampler3D                = ISampler<texture::target::D3>;
pub type ISampler2DMS              = ISampler<texture::target::D2MultiSample>;
pub type ISampler2DMSArray         = ISampler<texture::target::D2MultiSampleArray>;
pub type ISampler2DRect            = ISampler<texture::target::Rectangle>;
pub type ISampler2DRectShadow      = ISampler<Shadow<texture::target::Rectangle>>;
pub type ISampler2DCube            = ISampler<texture::target::CubeMap>;
pub type ISampler2DCubeShadow      = ISampler<Shadow<texture::target::CubeMap>>;
pub type ISampler2DCubeArray       = ISampler<texture::target::CubeMapArray>;
pub type ISampler2DCubeArrayShadow = ISampler<Shadow<texture::target::CubeMapArray>>;
pub type ISamplerBuffer            = ISampler<texture::target::Buffer>;


type USampler<Target> = GSampler<Target, u32>;

pub type USampler1D                = USampler<texture::target::D1>;
pub type USampler1DShadow          = USampler<Shadow<texture::target::D1>>;
pub type USampler1DArray           = USampler<texture::target::D1Array>;
pub type USampler1DArrayShadow     = USampler<Shadow<texture::target::D1Array>>;
pub type USampler2D                = USampler<texture::target::D2>;
pub type USampler2DShadow          = USampler<Shadow<texture::target::D2>>;
pub type USampler2DArray           = USampler<texture::target::D2Array>;
pub type USampler2DArrayShadow     = USampler<Shadow<texture::target::D2Array>>;
pub type USampler3D                = USampler<texture::target::D3>;
pub type USampler2DMS              = USampler<texture::target::D2MultiSample>;
pub type USampler2DMSArray         = USampler<texture::target::D2MultiSampleArray>;
pub type USampler2DRect            = USampler<texture::target::Rectangle>;
pub type USampler2DRectShadow      = USampler<Shadow<texture::target::Rectangle>>;
pub type USampler2DCube            = USampler<texture::target::CubeMap>;
pub type USampler2DCubeShadow      = USampler<Shadow<texture::target::CubeMap>>;
pub type USampler2DCubeArray       = USampler<texture::target::CubeMapArray>;
pub type USampler2DCubeArrayShadow = USampler<Shadow<texture::target::CubeMapArray>>;
pub type USamplerBuffer            = USampler<texture::target::Buffer>;

pub trait Binding<Target, const N: usize>
where 
    Target: texture::Target,
{ }


