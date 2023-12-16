use super::prelude::*;
use super::program::{CompiledShader, parameters};
use crate::object::resource::Allocator;
use crate::prelude::*;
use crate::{gl_call, impl_const_super_trait};
use crate::glsl;
use gl::types::GLenum;
use std::borrow::BorrowMut;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use crate::target::shader;

use crate::object::resource;
use thiserror;

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
    pub msg: String,
}

impl CompilationError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

struct ShaderSemantics<T, C>
where
    T: shader::Target,
    C: CompilationStatus,
{
    _target: PhantomData<T>,
    _compilation_status: PhantomData<C>,
}

impl<T, C> Default for ShaderSemantics<T, C>
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

type ShaderObject<T> = Object<ShaderAllocator<T>>;

pub struct Shader<T, C = Uncompiled>
where
    T: shader::Target,
    C: CompilationStatus,
{
    pub(crate) object: ShaderObject<T>,
    _semantics: ShaderSemantics<T, C>,
}

impl<T, C> Default for Shader<T, C>
where
    T: shader::Target,
    C: CompilationStatus,
{
    fn default() -> Self {
        Self {
            object: Default::default(),
            _semantics: Default::default(),
        }
    }
}

impl<T> Shader<T, Uncompiled>
where
    T: shader::Target,
{
    pub fn create() -> Self {
        Self::default()
    }

    /// Add source for shader.
    pub fn source(&self, sources: &[&str]) -> &Self {
        let pointers: Vec<_> = sources.iter().map(|s| s.as_ptr()).collect();
        let lengths: Vec<_> = sources.iter().map(|s| s.len()).collect();

        gl_call! {
            #[panic]
            unsafe {
                gl::ShaderSource(
                    self.object.name(),
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
                gl::GetShaderiv(self.object.name(), param as _, output);
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

    unsafe fn retype(self) -> Shader<T, Compiled> {
        Shader::<_, _> {
            object: self.object,
            _semantics: ShaderSemantics {
                _target: PhantomData,
                _compilation_status: PhantomData,
            },
        }
    }

    pub fn compile(self) -> Result<Shader<T, Compiled>, CompilationError> {
        gl_call! {
            #[propagate]
            unsafe {
                gl::CompileShader(self.object.name())
            }
        };
        self.info_log().map_or(
            // SAFETY: we just checked if shader compiled successfully
            Ok(unsafe { self.retype() }),
            |msg| Err(CompilationError { msg }),
        )
    }
}

impl<T> Shader<T, Compiled>
where
    T: shader::Target,
{
    pub fn into_main(self) -> Main<T, (), ()> {
        Main(self, PhantomData, PhantomData)
    }

    pub fn into_shared(self) -> Shared<T> {
        Shared(self)
    }
}

pub trait TargetProvider {
    type Target: shader::Target;
}

// todo impl From<CompiledShader<T>>
/// Shader that contains entry point for the stage
pub struct Main<T, I, O>(pub(crate) CompiledShader<T>, PhantomData<I>, PhantomData<O>)
where
    T: shader::Target,
    I: parameters::Parameters,
    O: parameters::Parameters,
;

impl<T, I, O> Main<T, I, O>
where
    T: shader::Target,
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub fn input<NI>(self) -> Main<T, (I, NI), O>
    where
        NI: glsl::types::Type,
    {
        let Self(shader, ..) = self;
        Main(shader, PhantomData, PhantomData)
    }

    pub fn output<NO>(self) -> Main<T, I, (O, NO)>
    where
        NO: glsl::types::Type,
    {
        let Self(shader, ..) = self;
        Main(shader, PhantomData, PhantomData)
    }
}

impl<T, I, O> std::ops::Deref for Main<T, I, O>
where
    T: shader::Target,
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    type Target = CompiledShader<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// todo impl From<CompiledShader<T>>
pub struct Shared<T>(pub(crate) CompiledShader<T>)
where
    T: shader::Target,
;

impl<T> std::ops::Deref for Shared<T>
where
    T: shader::Target,
{
    type Target = CompiledShader<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}