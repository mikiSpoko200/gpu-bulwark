use crate::constraint;
use crate::glsl;
use crate::glsl::Const;
use crate::impl_target;
use crate::mode;

/// Buffer object target types.
#[sealed::sealed]
#[hi::marker]
pub(crate) trait Target: crate::target::Target {}

#[macro_export]
#[allow(unused)]
macro_rules! impl_target {
    ($target_type:ty as $gl_target_ident: ident) => {
        impl $crate::target::Target for $target_type {
            const VALUE: u32 = glb::$gl_target_ident;
        }
        impl $crate::gl::target::buffer::Target for $target_type { }
    };
}

#[hi::mark(mode::Validation)]
pub enum Array { }

#[hi::mark(mode::Validation)]
pub enum AtomicCounter { }

#[hi::mark(mode::Validation)]
pub enum CopyRead { }

#[hi::mark(mode::Validation)]
pub enum CopyWrite { }

#[hi::mark(mode::Validation)]
pub enum DispatchIndirect { }

#[hi::mark(mode::Validation)]
pub enum DrawIndirect { }

#[hi::mark(mode::Validation)]
pub enum ElementArray { }

#[hi::mark(mode::Validation)]
pub enum PixelPack { }

#[hi::mark(mode::Validation)]
pub enum PixelUnpack { }

#[hi::mark(mode::Validation)]
pub enum Query { }

#[hi::mark(mode::Validation)]
pub enum ShaderStorage { }

#[hi::mark(mode::Validation)]
pub enum Texture { }

#[hi::mark(mode::Validation)]
pub enum TransformFeedback { } 

#[hi::mark(mode::Validation)]
pub enum Uniform { }

impl_target!{ Array as ARRAY_BUFFER }
impl_target!{ AtomicCounter as ATOMIC_COUNTER_BUFFER }
impl_target!{ CopyRead as COPY_READ_BUFFER }
impl_target!{ CopyWrite as COPY_WRITE_BUFFER }
impl_target!{ DispatchIndirect as DISPATCH_INDIRECT_BUFFER }
impl_target!{ DrawIndirect as DRAW_INDIRECT_BUFFER }
impl_target!{ ElementArray as ELEMENT_ARRAY_BUFFER }
impl_target!{ PixelPack as PIXEL_PACK_BUFFER }
impl_target!{ PixelUnpack as PIXEL_UNPACK_BUFFER }
impl_target!{ Query as QUERY_BUFFER }
impl_target!{ ShaderStorage as SHADER_STORAGE_BUFFER }
impl_target!{ Texture as TEXTURE_BUFFER }
impl_target!{ TransformFeedback as TRANSFORM_FEEDBACK_BUFFER }
impl_target!{ Uniform as UNIFORM_BUFFER }

pub mod alias {
    use crate::utils::Disjoint;
    use crate::glsl::{self, valid};

    pub trait ArrayType: Disjoint + glsl::alias::TransparentType { }

    trait ArrayTypeDH<Discriminant>: glsl::alias::TransparentType { }

    impl<T> ArrayTypeDH<glsl::valid::Scalar> for T
    where 
        T: valid::ForScalar
        + Disjoint<Discriminant = glsl::valid::Scalar>
    { }

    impl<T> ArrayTypeDH<glsl::valid::Vector> for T
    where 
        T: valid::ForVector
        + Disjoint<Discriminant = glsl::valid::Vector> 
    { }

    impl<T> ArrayType for T where T: glsl::alias::TransparentType + ArrayTypeDH<T::Subtype> { }
}

pub mod valid {
    fn test<T>() where T: super::alias::ArrayType { }

    fn dupa() {
        test::<f32>();
        test::<crate::glsl::GVec<f32, 2>>();
        test::<crate::glsl::Mat<f32, 2, 3>>();
    }

    /// GLSL types valid for use with buffer objects binded to `Array` target.
    #[hi::marker]
    pub trait ForArray: super::glsl::alias::VectorType { }
}

impl<T, const SIZE: usize> valid::ForArray for glsl::GVec<T, SIZE>
where
    T: glsl::Type + constraint::Valid<Vector>,
    Const<SIZE>: constraint::Valid<Vector>,
{
}
