use super::prelude::*;
use crate::object::resource::Allocator;
use crate::prelude::*;
use crate::{gl_call, impl_const_super_trait};
use gl::types::GLenum;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use crate::object::resource;
use thiserror;

pub trait Target: crate::target::Target {}

/// Zero-sized struct that represents Vertex Shader stage.
pub struct Vertex;

pub mod tesselation {
    /// Zero-sized struct that represents Tesselation Control Shader stage.
    pub struct Control;

    /// Zero-sized struct that represents Tesselation Evaluation Shader stage.
    pub struct Evaluation;
}

/// Zero-sized struct that represents Geometry Shader stage.
pub struct Geometry;

/// Zero-sized struct that represents Fragment Shader stage.
pub struct Fragment;

/// Zero-sized struct that represents Compute Shader stage.
pub struct Compute;

impl_const_super_trait!(Target for Vertex, gl::VERTEX_SHADER);
impl_const_super_trait!(Target for tesselation::Control, gl::TESS_CONTROL_SHADER);
impl_const_super_trait!(Target for tesselation::Evaluation, gl::TESS_EVALUATION_SHADER);
impl_const_super_trait!(Target for Geometry, gl::GEOMETRY_SHADER);
impl_const_super_trait!(Target for Fragment, gl::FRAGMENT_SHADER);
impl_const_super_trait!(Target for Compute, gl::COMPUTE_SHADER);

pub trait CompilationStatus {}

#[derive(Default)]
pub struct Uncompiled;
impl CompilationStatus for Uncompiled {}

pub struct Compiled;
impl CompilationStatus for Compiled {}

#[repr(u32)]
pub enum QueryParam {
    ShaderType = gl::SHADER_TYPE,
    DeleteStatus = gl::DELETE_STATUS,
    CompileStatus = gl::COMPILE_STATUS,
    InfoLogLength = gl::INFO_LOG_LENGTH,
    ShaderSourceLength = gl::SHADER_SOURCE_LENGTH,
}

#[derive(thiserror::Error, Debug)]
#[error("shader compilation failed {msg}")]
pub struct CompilationError {
    msg: String,
}

impl CompilationError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

#[derive(Default)]
struct ShaderSemantics<S, C>
where
    S: Target,
    C: CompilationStatus,
{
    _stage: PhantomData<S>,
    _compilation_status: PhantomData<C>,
}

struct ShaderAllocator<S>;

unsafe impl<T: Target> Allocator for ShaderAllocator<T> {
    fn allocate(names: &mut [Name]) {
        for name in names {
            *name = unsafe { gl::CreateShader(T::VALUE) };
            // TODO: Check for errors
        }
    }

    fn free(names: &[Name]) {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for name in names {
            unsafe { gl::DeleteShader(*name) };
            // TODO: Check for errors
        }
    }
}

pub type ShaderObject<T> = Object<ShaderAllocator<T>>;

#[derive(Default)]
pub struct Shader<S, C = Uncompiled>
where
    S: Target,
    C: CompilationStatus,
{
    object: Object<Self>,
    _semantics: ShaderSemantics<S, C>
}


impl<S> Shader<S, Uncompiled>
where
    S: Target,
{
    pub fn create() -> Self {
        Self::default()
    }

    /// Add source for shader.
    pub fn source(&self, sources: &[&str]) -> &Self {
        let pointers: Vec<_> = sources.iter()
            .map(|s| s.as_ptr())
            .collect();
        let lengths: Vec<_> = sources.iter()
            .map(|s| s.len())
            .collect();

        gl_call! {
            #[panic]
            unsafe {
                gl::ShaderSource(
                    self.base.name,
                    sources.len() as _,
                    pointers.as_ptr() as _,
                    lengths.as_ptr() as _
                );
            }
        }
        self
    }

    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
            unsafe {
                gl::GetShaderiv(self.base.name, param as _, output);
            }
        }
    }

    pub fn info_log(&self) -> Option<String> {
        let mut log_size = 0;
        self.query(QueryParam::InfoLogLength, &mut log_size);
        (log_size > 0).then(|| {
            let mut buffer: Vec<u8> = Vec::with_capacity(log_size as _);
            let mut actual_length = 0;
            gl_call! {
                #[panic]
                // SAFETY: All values passed are valid
                // todo: notes on error situations
                unsafe {
                    gl::GetShaderInfoLog(
                        self.base.name,
                        buffer.capacity() as _,
                        &mut actual_length as *mut _,
                        buffer.as_mut_ptr() as _
                    )
                }
            }
            // GetShaderInfoLog does not account for null terminator in returned length.
            // SAFETY: nothing will panic here so it's safe to set length.
            unsafe {
                buffer.set_len((actual_length + 1) as _);
            }
            // SAFETY: todo will shader compiler should emmit valid ascii?
            unsafe { String::from_utf8_unchecked(buffer) }
        })
    }

    unsafe fn retype(self) -> Shader<S, Compiled> {
        Self {
            object: self.object,
            _semantics: Default::default()
        }
    }

    pub fn compile(self) -> Result<Shader<S, Compiled>, CompilationError> {
        gl_call! {
            #[propagate]
            unsafe {
                gl::CompileShader(self.base.name)
            }
        };
        self
            .info_log()
            .map_or(
                // SAFETY: we just checked if shader compiled successfully
                Ok(unsafe { self.retype() }),
                |msg| Err(CompilationError { msg })
            )
    }
}

impl<S> Shader<S, Compiled>
where
    S: Target,
{
    pub fn inputs() {}
    pub fn outputs() {}
}
