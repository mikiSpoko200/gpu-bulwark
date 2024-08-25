use std::ops::{Range, RangeBounds};

use crate::gl::object;
use crate::gl::target::Target;
use crate::glsl;
use crate::prelude::internal::*;
use crate::gl;
use gl::texture;
use std::ops::RangeInclusive;
use texture::target;
use texture::storage;

use super::image;
use super::pixel;
use super::target::Dimensionality;
use super::TextureObject;

pub struct Immutable<T>(PhantomData<T>) where T: texture::Target + marker::Internal;

pub struct Mutable<T>(PhantomData<T>) where T: texture::Target + marker::Internal;

pub mod marker {
    use super::*;

    mod private {
        use super::texture;

        pub trait Internal: texture::Target  { }
    }
    pub(in crate::gl::texture) use private::*;

    

    pub trait Storage: Kind + AllocatorDispatch { }

    /// Type which represent different types of storage that texture can use.
    /// NOTE: They do **not** represent actual storage yet only its origin / mutability.
    pub trait Kind {
        type Target: texture::Target;
    }

    impl<T: texture::Target> Kind for Immutable<T> where T: texture::Target + Internal {
        type Target = T;
    }
    impl<T: texture::Target + Internal> Storage for Immutable<T> where Self: AllocatorDispatch { }

    impl<T: texture::Target> Kind for Mutable<T> where T: texture::Target + Internal {
        type Target = T;
    }
    impl<T: texture::Target + Internal> Storage for Mutable<T> where Self: AllocatorDispatch { }

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
    InternalFormat: image::marker::Format,
{
    /// Type-state parameter that controls what operations are available for the storage.
    kind: PhantomData<Kind>,
    /// Information about memory layout of the texture.
    layout: Layout<Target, InternalFormat, CONTAINS_MIPMAPS>
}

impl<Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool, const DIM: usize> Storage<Target, Kind, InternalFormat, CONTAINS_MIPMAPS>
where
    Const<DIM>: texture::valid::TextureDim,
    Target: texture::Target + Dimensionality<Dimensions = [usize; DIM]>,
    Kind: marker::Storage<Target=Target>,  
    InternalFormat: image::marker::Format,
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
    Kind: marker::Storage<Target=D1Target>,
    InternalFormat: image::marker::Format,
{
    pub fn sub_image_1d<
        Channels: pixel::channels::Channels,
        Pixel: pixel::Pixel<Components = Channels::Components, Kind = <InternalFormat::Output as glsl::sampler::Output>::Kind>,
    >(
        &mut self,
        _: &gl::object::Bind<texture::TextureObject<D1Target>>, 
        x_range: impl std::ops::RangeBounds<usize>, 
        pixels: &[Pixel]
    )
    where
        Channels: pixel::valid::ForImageBaseFormat<InternalFormat::BaseFormat>,
        (Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind): pixel::FormatToken,
    {
        let [width] = self.layout.dimensions;
        let (start, end) = Self::range(width, x_range);
        if end > width {
            panic!("range {start}..={end} extends outside of texture width");
        }
        println!("{}, {}", 
            <(Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind) as pixel::FormatToken>::ID,
            Pixel::type_token()
        );
        let length = end - start;
        gl::call! {
            [panic]
            unsafe {
                glb::TexSubImage1D(
                    D1Target::ID,
                    0,
                    start as _,
                    length as _,
                    <(Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind) as pixel::FormatToken>::ID,
                    Pixel::type_token(),
                    pixels.as_ptr() as *const _,
                );
            }
        }
    }
}

impl<D2Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool> Storage<D2Target, Kind, InternalFormat, CONTAINS_MIPMAPS>
where
    D2Target: texture::Target<Dimensions = [usize; 2]>,
    Kind: marker::Storage<Target=D2Target>,
    InternalFormat: image::marker::Format,
{
    pub fn sub_image_2d<
        Channels: pixel::channels::Channels,
        Pixel: pixel::Pixel<Components = Channels::Components, Kind = <InternalFormat::Output as glsl::sampler::Output>::Kind>,
    >(
        &mut self,
        _: &gl::object::Bind<texture::TextureObject<D2Target>>,
        x_range: impl std::ops::RangeBounds<usize>, 
        y_range: impl std::ops::RangeBounds<usize>,
        pixels: &mut [Pixel]
    )
    where
        Channels: pixel::valid::ForImageBaseFormat<InternalFormat::BaseFormat>,
        (Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind): pixel::FormatToken,
    {
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

        println!("{}, {}, {}, {}", 
        <(Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind) as pixel::FormatToken>::ID,
        glb::RGB,
        Pixel::type_token(),
        glb::UNSIGNED_BYTE,
    );

        gl::call! {
            [panic]
            {
                println!("foo");
            }
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
                    <(Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind) as pixel::FormatToken>::ID,
                    Pixel::type_token(),
                    pixels.as_ptr() as *const _,
                );
            }
        }
    }
}

impl<D3Target, Kind, InternalFormat, const CONTAINS_MIPMAPS: bool> Storage<D3Target, Kind, InternalFormat, CONTAINS_MIPMAPS>
where
    D3Target: texture::Target<Dimensions = [usize; 3]>,
    Kind: marker::Storage<Target=D3Target>,
    InternalFormat: image::marker::Format,
{
    pub fn sub_image_3d<
        Channels: pixel::channels::Channels,
        Pixel: pixel::Pixel<Components = Channels::Components, Kind = <InternalFormat::Output as glsl::sampler::Output>::Kind>,
    >(
        &mut self,
        _: &gl::object::Bind<texture::TextureObject<D3Target>>,
        x_range: impl std::ops::RangeBounds<usize>,
        y_range: impl std::ops::RangeBounds<usize>,
        z_range: impl std::ops::RangeBounds<usize>,
        pixels: &[Pixel]
    )
    where
        Channels: pixel::valid::ForImageBaseFormat<InternalFormat::BaseFormat>,
        (Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind): pixel::FormatToken,
    {
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
                    <(Channels, <InternalFormat::Output as glsl::sampler::Output>::Kind) as pixel::FormatToken>::ID,
                    Pixel::type_token(),
                    pixels.as_ptr() as *const _,
                );
            }
        }
    }
}

impl<D1Target, Kind, InternalFormat> Storage<D1Target, Kind, InternalFormat, false>
where
    D1Target: texture::Target<Dimensions = [usize; 1]>,
    Kind: marker::Storage<Target=D1Target, Signature = signature::Storage1D>,
    InternalFormat: image::marker::Format,
{
    pub fn storage_1d(_: &gl::object::Bind<TextureObject<D1Target>>, width: usize) -> Self {
        gl::call! {
            [panic]
            unsafe {
                Kind::ALLOCATOR(D1Target::ID, 1, InternalFormat::ID, width as _);
            }
        }
        Self {
            kind: PhantomData,
            layout: Layout {
                target: PhantomData,
                internal_format: PhantomData,
                dimensions: [width],
            },
        }
    }
}

impl<D2Target, Kind, InternalFormat> Storage<D2Target, Kind, InternalFormat, false>
where
    D2Target: texture::Target<Dimensions = [usize; 2]>,
    Kind: marker::Storage<Target=D2Target, Signature = signature::Storage2D>,
    InternalFormat: image::marker::Format,
{
    pub fn storage_2d(_: &object::Bind<TextureObject<D2Target>>, width: usize, height: usize) -> Self {
        gl::call! {
            [panic]
            unsafe {
                Kind::ALLOCATOR(D2Target::ID, 1, InternalFormat::ID, width as _, height as _);
            }
        }
        Self {
            kind: PhantomData,
            layout: Layout {
                target: PhantomData,
                internal_format: PhantomData,
                dimensions: [width, height],
            },
        }
    }
}

impl<D3Target, Kind, InternalFormat> Storage<D3Target, Kind, InternalFormat, false>
where
    D3Target: texture::Target<Dimensions = [usize; 3]>,
    Kind: marker::Storage<Target=D3Target, Signature = signature::Storage3D>,
    InternalFormat: image::marker::Format,
{
    pub fn storage_3d(_: &object::Bind<TextureObject<D3Target>>, width: usize, height: usize, depth: usize) -> Self {
        gl::call! {
            [panic]
            unsafe {
                Kind::ALLOCATOR(D3Target::ID, 1, InternalFormat::ID, width as _, height as _, depth as _);
            }
        }
        Self {
            kind: PhantomData,
            layout: Layout {
                target: PhantomData,
                internal_format: PhantomData,
                dimensions: [width, height, depth],
            },
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
 	// GLint xoffset``,
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
pub trait AllocatorDispatch {
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

impl<GL> Dimensionality for gl::Buffer<texture::Buffer, GL> {
    type Dimensions = [usize; 1];
}

impl<GL> AllocatorDispatch for gl::Buffer<texture::Buffer, GL> {
    type Signature = signature::Buffer;
    const ALLOCATOR: Self::Signature = glb::TexBuffer;
}
