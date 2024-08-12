use std::marker::PhantomData;
use crate::gl;
use gl::texture;
use gl::buffer;
use crate::gl::texture::target;

pub trait Kind { }

#[hi::mark(Kind)]
pub struct Immutable<T>(PhantomData<T>) where T: texture::Target;

#[hi::mark(Kind)]
pub struct Mutable<T>(PhantomData<T>) where T: texture::Target;


pub trait Storage<T> where T: texture::Target {
    /// Data type of index and dimensions of storage
    type Index;
}

impl<T> Storage<T> for Immutable<T> where T: texture::Target {
    type Index = T::Index;
}


pub mod signature {
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
}

/// Map storage kinds to functions that allocate them
pub trait AllocatorDispatch<K>: texture::Target where K: Kind {
    /// Type specific allocation routine signature.
    type Signature;
    /// Pointer to OpenGL texture storage allocation routine.
    const ALLOCATOR: Self::Signature;
}

macro_rules! dispatch_allocator {
    ([immutable] $type: ty => $function: path: $signature: path) => {
        impl AllocatorDispatch<Immutable> for $type {
            type Signature = $signature;
            const ALLOCATOR: Self::Signature = $function;
        }
    };
    ([mutable] $type: ty => $function: path: $signature: path) => {
        impl AllocatorDispatch<Mutable> for $type {
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

impl AllocatorDispatch<Buffer> for target::Buffer {
    type Signature = signature::Buffer;
    const ALLOCATOR: Self::Signature = glb::TexBuffer;
}
