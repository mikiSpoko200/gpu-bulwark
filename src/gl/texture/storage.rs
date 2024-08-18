use std::ops::{Range, RangeBounds};

use crate::gl::target::Target;
use crate::prelude::internal::*;
use crate::gl;
use gl::texture;
use std::ops::RangeInclusive;
use texture::target;
use texture::storage;

use super::target::Dimensionality;

pub struct Immutable<T>(PhantomData<T>) where T: texture::Target + marker::Internal;

pub struct Mutable<T>(PhantomData<T>) where T: texture::Target + marker::Internal;

pub mod marker {
    use super::*;

    pub(in crate::gl::texture) trait Internal: texture::Target  { }

    /// Type which represent different types of storage that texture can use.
    /// NOTE: They do **not** represent actual storage yet only its origin / mutability.
    pub trait Kind {
        type Target: texture::Target;
    }

    impl<T: texture::Target> Kind for Immutable<T> where T: texture::Target + Internal {
        type Target = T;
    }

    impl<T: texture::Target> Kind for Mutable<T> where T: texture::Target + Internal {
        type Target = T;
    }

    impl<GL> Kind for gl::Buffer<texture::Buffer, GL> {
        type Target = texture::Buffer;
    }
}

/// Memory layout of texture storage which includes its dimensions and format.
struct Layout<Target, InternalFormat, const CONTAINS_MIPMAPS: bool>
where 
    Target: texture::Target,
{
    target: PhantomData<Target>,
    internal_format: PhantomData<InternalFormat>,
    dimensions: Target::Dimensions,
}

/// Abstraction of texture storage.
/// 
/// Storage will be injected into a texture and specific storage types can be constructed with appropriate free functions.
pub struct Storage<Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool>
where
    Target: texture::Target,
    Kind: marker::Kind<Target=Target>,
{
    /// Type-state parameter that controls what operations are available for the storage.
    kind: PhantomData<Kind>,
    /// Information about memory layout of the texture.
    layout: Layout<Target, InternalFormat, CONTAINS_MIPMAPS>
}

pub trait Pixel {
    const FORMAT: u32;
    const TYPE: u32;
}

impl<Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool, const DIM: usize> Storage<Target, Kind, InternalFormat, CONTAINS_MIPMAPS>
where
    Const<DIM>: texture::valid::TextureDim,
    Target: texture::Target + Dimensionality<Dimensions = [usize; DIM]>,
    Kind: marker::Kind<Target=Target>,  
{
    fn range(len: usize, span: impl std::ops::RangeBounds<usize>) -> (usize, usize) {
        let start = match span.start_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n + 1,
            std::ops::Bound::Unbounded => 0,
        };
        let end = match span.end_bound() {
            std::ops::Bound::Included(&n) => n,
            std::ops::Bound::Excluded(&n) => n - 1,
            std::ops::Bound::Unbounded => len,
        };
        (start, end)
    }
}

impl<D1Target, Kind, InternalFormat> Storage<D1Target, Kind, InternalFormat, false>
where
    D1Target: texture::Target<Dimensions = [usize; 1]>,
    Kind: marker::Kind<Target=D1Target>,
{
    pub fn sub_image_1d<P: Pixel>(&mut self, x_range: impl std::ops::RangeBounds<usize>, pixels: &[P]) {
        let [width] = self.layout.dimensions;
        let (start, end) = Self::range(width, x_range);
        if end > width {
            panic!("range {start}..={end} extends outside of texture width");
        }
        let length = end - start;
        gl::call! {
            [panic]
            unsafe {
                glb::TexSubImage1D(
                    D1Target::ID,
                    0,
                    start as _,
                    length as _,
                    P::FORMAT,
                    P::TYPE,
                    pixels.as_ptr() as *const _,
                );
            }
        }
    }
}

impl<D2Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool> Storage<D2Target, Kind, InternalFormat, CONTAINS_MIPMAPS>
where
    D2Target: texture::Target<Dimensions = [usize; 2]>,
    Kind: marker::Kind<Target=D2Target>,
{
    pub fn sub_image_2d<P: Pixel>(
        &mut self, 
        x_range: impl std::ops::RangeBounds<usize>, 
        y_range: impl std::ops::RangeBounds<usize>,
        pixels: &[P]
    ) {
        let [width, height] = self.layout.dimensions;
        let (x_start, x_end) = Self::range(width, x_range);
        let (y_start, y_end) = Self::range(height, y_range);

        let x_length = x_end - x_start;
        let y_length = y_end - y_start;

        if x_end > width {
            panic!("sub image width range {x_start}..={x_end} extends out of bounds");
        }
        if y_end > height {
            panic!("sub image height range {y_start}..={y_end} extends out of bounds");
        }

        gl::call! {
            [panic]
            unsafe {
                glb::TexSubImage2D(
                    D2Target::ID,
                    0,
                    x_start as _,
                    y_start as _,
                    x_length as _,
                    y_length as _,
                    P::FORMAT,
                    P::TYPE,
                    pixels.as_ptr() as *const _,
                );
            }
        }
    }
}

impl<D3Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool> Storage<D3Target, Kind, InternalFormat, CONTAINS_MIPMAPS>
where
    D3Target: texture::Target<Dimensions = [usize; 3]>,
    Kind: marker::Kind<Target=D3Target>,
{
    pub fn sub_image_3d<P: Pixel>(
        &mut self, 
        x_range: impl std::ops::RangeBounds<usize>,
        y_range: impl std::ops::RangeBounds<usize>,
        z_range: impl std::ops::RangeBounds<usize>,
        pixels: &[P]
    ) {
        let [width, height, depth] = self.layout.dimensions;
        let (x_start, x_end) = Self::range(width, x_range);
        let (y_start, y_end) = Self::range(height, y_range);
        let (z_start, z_end) = Self::range(depth, z_range);

        if x_end > width {
            panic!("sub image width range {x_start}..={x_end} extends out of bounds");
        }
        if y_end > height {
            panic!("sub image height range {y_start}..={y_end} extends out of bounds");
        }
        if z_end > depth {
            panic!("sub image depth range {z_start}..={z_end} extends out of bounds");
        }

        let x_length = x_end - x_start;
        let y_length = y_end - y_start;
        let z_length = z_end - z_start;

        gl::call! {
            [panic]
            unsafe {
                glb::TexSubImage3D(
                    D3Target::ID,
                    0,
                    x_start as _,
                    y_start as _,
                    z_start as _,
                    x_length as _,
                    y_length as _,
                    z_length as _,
                    P::FORMAT,
                    P::TYPE,
                    pixels.as_ptr() as *const _,
                );
            }
        }
    }
}

/// View into texture image that supports indexing.
/// 
/// Range parameter controls the scope of subimage which can be progressively refined.
// pub struct SubImage<Target, InternalFormat, Range>(Layout<Target, InternalFormat>, PhantomData<Range>) where Target: texture::Target;

pub mod signature {
    use crate::gl::texture;

    pub type Storage1D = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32
    ) -> ();
    pub type Storage2D = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32, /* height */ i32
    ) -> ();
    pub type Storage3D = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32, /* height */ i32, /* depth */ i32
    ) -> ();
    pub type Storage2DMultisample = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32, /* height */ i32,
        /* fixed sample locations */ u8
    ) -> ();
    pub type Storage3DMultisample = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32, /* height */ i32, /* depth */ i32,
        /* fixed sample locations */ u8
    ) -> ();

    pub type Image1D = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ i32,
        /* width */ i32,
        /* boarder */ i32,
        /* format */ u32,
        /* type */ u32,
        /* data */ *const std::ffi::c_void
    ) -> ();
    pub type Image2D = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ i32,
        /* width */ i32,
        /* height */ i32,
        /* boarder */ i32,
        /* format */ u32,
        /* type */ u32,
        /* data */ *const std::ffi::c_void
    ) -> ();
    pub type Image3D = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ i32,
        /* width */ i32,
        /* height */ i32,
        /* depth */ i32,
        /* boarder */ i32,
        /* format */ u32, 
        /* type */ u32, 
        /* data */ *const std::ffi::c_void
    ) -> ();
    pub type Image2DMultisample = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32,
        /* height */ i32,
        /* fixed sample locations */ u8
    ) -> ();
    pub type Image3DMultisample = unsafe fn(
        /* Target */ u32,
        /* levels */ i32,
        /* internal format */ u32,
        /* width */ i32,
        /* height */ i32,
        /* depth */ i32,
        /* fixed sample locations */ u8
    ) -> ();

    pub type Buffer = unsafe fn(/* Target */ u32, /* internal format */ u32, /* buffer */ u32);

    // GLenum target,
 	// GLint level,
 	// GLint xoffset,
 	// GLsizei width,
 	// GLenum format,
 	// GLenum type,
 	// const void * pixels
    pub type SubImage1D = unsafe fn(
        /* target */ u32,
        /* level */ i32,
        /* xoffset */ i32,
        /* width */ i32,
        /* format */ u32,
        /* type */ u32,
        /* pixels */ *const std::ffi::c_void,
    );
}

/// Map storage kinds to functions that allocate them
pub trait AllocatorDispatch: Dimensionality {
    /// Type specific allocation routine signature.
    type Signature;
    /// Pointer to OpenGL texture storage allocation routine.
    const ALLOCATOR: Self::Signature;
}

macro_rules! dispatch_allocator {
    ([immutable] $target:ty => $function:path: $signature:path) => {
        impl AllocatorDispatch for Immutable<$target> {
            type Signature = $signature;
            const ALLOCATOR: Self::Signature = $function;
        }
    };
    ([mutable] $target:ty => $function:path: $signature:path) => {
        impl AllocatorDispatch for Mutable<$target> {
            type Signature = $signature;
            const ALLOCATOR: Self::Signature = $function;
        }
    };
}

dispatch_allocator! { [immutable] target::D1 => glb::TexStorage1D: signature::Storage1D }

dispatch_allocator! { [immutable] target::D2        => glb::TexStorage2D: signature::Storage2D }
dispatch_allocator! { [immutable] target::Rectangle => glb::TexStorage2D: signature::Storage2D }
dispatch_allocator! { [immutable] target::CubeMap   => glb::TexStorage2D: signature::Storage2D }
dispatch_allocator! { [immutable] target::D1Array   => glb::TexStorage2D: signature::Storage2D }

dispatch_allocator! { [immutable] target::D3           => glb::TexStorage3D: signature::Storage3D }
dispatch_allocator! { [immutable] target::D2Array      => glb::TexStorage3D: signature::Storage3D }
dispatch_allocator! { [immutable] target::CubeMapArray => glb::TexStorage3D: signature::Storage3D }

dispatch_allocator! { [immutable] target::D2MultiSample => glb::TexStorage2DMultisample: signature::Storage2DMultisample }

dispatch_allocator! { [immutable] target::D2MultiSampleArray => glb::TexStorage3DMultisample: signature::Storage3DMultisample }


dispatch_allocator! { [mutable] target::D1 => glb::TexImage1D: signature::Image1D }

dispatch_allocator! { [mutable] target::D2        => glb::TexImage2D: signature::Image2D }
dispatch_allocator! { [mutable] target::Rectangle => glb::TexImage2D: signature::Image2D }
dispatch_allocator! { [mutable] target::CubeMap   => glb::TexImage2D: signature::Image2D }
dispatch_allocator! { [mutable] target::D1Array   => glb::TexImage2D: signature::Image2D }

dispatch_allocator! { [mutable] target::D3           => glb::TexImage3D: signature::Image3D }
dispatch_allocator! { [mutable] target::D2Array      => glb::TexImage3D: signature::Image3D }
dispatch_allocator! { [mutable] target::CubeMapArray => glb::TexImage3D: signature::Image3D }

dispatch_allocator! { [mutable] target::D2MultiSample => glb::TexImage2DMultisample: signature::Image2DMultisample }

dispatch_allocator! { [mutable] target::D2MultiSampleArray => glb::TexImage3DMultisample: signature::Image3DMultisample }

impl<GL> AllocatorDispatch for GlSurface::Buffer<texture::Buffer, GL> {
    type Signature = signature::Buffer;
    const ALLOCATOR: Self::Signature = glb::TexBuffer;
}
