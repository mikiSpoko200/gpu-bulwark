use std::marker::PhantomData;

use crate::glsl::parameters;
use super::target as shader;
use super::target;
use super::{CompilationStatus, Compiled, Uncompiled};
use super::{Main, Shared};
use crate::gl_call;
use crate::glsl;
use crate::gl::prelude::*;
use crate::gl::object::{Allocator, Bind};

use crate::gl::object;
use thiserror;

#[repr(u32)]
pub enum QueryParam {
    ShaderType = glb::SHADER_TYPE,
    DeleteStatus = glb::DELETE_STATUS,
    CompileStatus = glb::COMPILE_STATUS,
    InfoLogLength = glb::INFO_LOG_LENGTH,
    ShaderSourceLength = glb::SHADER_SOURCE_LENGTH,
}

struct ShaderPhantomData<T, C>
where
    T: shader::Target,
    C: CompilationStatus,
{
    _target: PhantomData<T>,
    _compilation_status: PhantomData<C>,
}

impl<T, C> Default for ShaderPhantomData<T, C>
where
    T: shader::Target,
    C: CompilationStatus,
{
    fn default() -> Self {
        Self {
            _target: Default::default(),
            _compilation_status: Default::default(),
        }
    }
}

/// Allocator for Shader Objects.
pub(crate) struct ShaderAllocator<T>(PhantomData<T>)
where
    T: shader::Target;

unsafe impl<T: shader::Target> Allocator for ShaderAllocator<T> {
    fn allocate(names: &mut [u32]) {
        for name in names {
            *name = unsafe { glb::CreateShader(T::VALUE) };
            // TODO: Check for errors
        }
    }

    fn free(names: &[u32]) {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for name in names {
            unsafe { glb::DeleteShader(*name) };
            // TODO: Check for errors
        }
    }
}

pub(super) type ShaderObject<T> = Object<ShaderAllocator<T>>;

pub struct Shader<T, C>
where
    T: super::shader::Target,
    C: super::CompilationStatus,
{
    pub(crate) object: ShaderObject<T>,
    _phantoms: ShaderPhantomData<T, C>,
}

pub type CompiledShader<T> = Shader<T, Compiled>;

pub type VertexShader = CompiledShader<target::Vertex>;
pub type TesselationControlShader = CompiledShader<target::tesselation::Control>;
pub type TesselationEvaluationShader = CompiledShader<target::tesselation::Evaluation>;
pub type GeometryShader = CompiledShader<target::Geometry>;
pub type FragmentShader = CompiledShader<target::Fragment>;
pub type ComputeShader = CompiledShader<target::Compute>;

impl<T> Default for Shader<T, Uncompiled>
where
    T: shader::Target,
{
    fn default() -> Self {
        Self::create()
    }
}

impl<T> Shader<T, Uncompiled>
where
    T: shader::Target,
{
    pub(super) fn create() -> Self {
        Self {
            object: Default::default(),
            _phantoms: Default::default(),
        }
    }

    /// Add source for shader.
    pub(super) fn source(&self, sources: &[&str]) -> &Self {
        let pointers: Vec<_> = sources.iter().map(|s| s.as_ptr()).collect();
        let lengths: Vec<_> = sources.iter().map(|s| s.len()).collect();

        gl_call! {
            #[panic]
            unsafe {
                glb::ShaderSource(
                    self.object.name(),
                    sources.len() as _,
                    pointers.as_ptr() as _,
                    lengths.as_ptr() as _
                );
            }
        }
        self
    }

    pub(super) unsafe fn retype_to_compiled(self) -> Shader<T, Compiled> {
        Shader::<_, _> {
            object: self.object,
            _phantoms: ShaderPhantomData {
                _target: PhantomData,
                _compilation_status: PhantomData,
            },
        }
    }

    pub(super) fn compile(self) -> Result<Shader<T, Compiled>, super::CompilationError> {
        gl_call! {
            #[propagate]
            unsafe {
                glb::CompileShader(self.object.name())
            }
        };
        self.info_log().map_or(
            // SAFETY: we just checked if shader compiled successfully
            Ok(unsafe { self.retype_to_compiled() }),
            |msg| Err(super::CompilationError { msg }),
        )
    }

    fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
            unsafe {
                glb::GetShaderiv(self.object.name(), param as _, output);
            }
        }
    }

    fn info_log(&self) -> Option<String> {
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
                    glb::GetShaderInfoLog(
                        self.object.name(),
                        buffer.capacity() as _,
                        &mut actual_length as *mut _,
                        buffer.as_mut_ptr() as _
                    )
                }
            }
            // GetShaderInfoLog does not account for null terminator in returned length.
            // SAFETY: nothing will panic here so it's safe to set length.
            unsafe {
                buffer.set_len(actual_length as _);
            }
            // SAFETY: todo will shader compiler should emmit valid ascii?
            let message = unsafe { String::from_utf8_unchecked(buffer) };
            message
        })
    }
}
