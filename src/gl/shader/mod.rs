use crate::hlist::lhlist;
use crate::prelude::internal::*;

pub mod lib;
pub mod main;
pub mod prelude;
pub mod target;

pub use target::Target;

use crate::glsl::{self, location};
use crate::hlist::indexed::rhlist;
use crate::prelude::*;
use crate::{gl, ts};
use gl::uniform;

pub(super) use lib::Lib;
pub(super) use main::Main;

use super::UniformBinding;

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

#[repr(u32)]
pub enum QueryParam {
    ShaderType = glb::SHADER_TYPE,
    DeleteStatus = glb::DELETE_STATUS,
    CompileStatus = glb::COMPILE_STATUS,
    InfoLogLength = glb::INFO_LOG_LENGTH,
    ShaderSourceLength = glb::SHADER_SOURCE_LENGTH,
}

struct ShaderState<C>
where
    C: ts::Compilation,
{
    _phantom: PhantomData<C>,
}

impl<C> Default for ShaderState<C>
where
    C: ts::Compilation,
{
    fn default() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

/// Allocator for Shader Objects.
pub(crate) struct ShaderAllocator<T>(PhantomData<T>)
where
    T: Target;

unsafe impl<T: Target> gl::object::Allocator for ShaderAllocator<T> {
    fn allocate(names: &mut [u32]) {
        for name in names {
            gl::call! {
                [panic]
                *name = unsafe { glb::CreateShader(T::VALUE) }
            }
        }
    }

    fn free(names: &[u32]) {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for name in names {
            gl::call! {
                [panic]
                unsafe { glb::DeleteShader(*name) }
            }
        }
    }
}

pub(super) type ShaderObject<T> = gl::object::ObjectBase<Shader>;

pub struct Shader<C, T, Decls>
where
    C: ts::Compilation,
    T: target::Target,
    Decls: uniform::bounds::Declarations,
{
    pub(in crate::gl) object: ShaderObject<T>,
    pub(in crate::gl) state: ShaderState<C>,
    pub(in crate::gl) uniform_declarations: uniform::Declarations<ts::Mutable, Decls>,
}

impl<T: Target> Default for Shader<ts::Uncompiled, T, ()> {
    fn default() -> Self {
        Self::create()
    }
}

impl<C, T, Decls> AsRef<ShaderObject<T>> for Shader<C, T, Decls>
where 
    C: ts::Compilation,
    T: Target,
    Decls: uniform::bounds::Declarations,
{
    fn as_ref(&self) -> &ShaderObject<T> {
        &self.object
    }
}

impl<T: Target> Shader<ts::Uncompiled, T, ()> {
    pub fn create() -> Self {
        Self {
            object: ShaderObject::default(),
            state: ShaderState::default(),
            uniform_declarations: uniform::Declarations::default(),
        }
    }
}

impl<T, Decls> Shader<ts::Uncompiled, T, Decls>
where
    T: Target,
    Decls: uniform::bounds::Declarations,
{
    /// Declare uniform variable used by this shader
    pub fn declare<U, const LOCATION: usize>(
        self,
        decl: UniformBinding<U, LOCATION>,
    ) -> Shader<ts::Uncompiled, T, (Decls, uniform::Declaration<U, LOCATION>)>
    where
        U: glsl::Uniform,
    {
        Shader {
            uniform_declarations: self
                .uniform_declarations
                .declare(uniform::Declaration::from(decl)),
            object: self.object,
            state: self.state,
        }
    }

    /// Add source for shader.
    pub fn source(&self, sources: &[&str]) -> &Self {
        let pointers: Vec<_> = sources.iter().map(|s| s.as_ptr()).collect();
        let lengths: Vec<_> = sources.iter().map(|s| s.len()).collect();

        gl::call! {
            [panic]
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

    fn retype_to_compiled(self) -> Shader<ts::Compiled, T, Decls> {
        Shader {
            object: self.object,
            state: ShaderState::default(),
            uniform_declarations: self.uniform_declarations,
        }
    }

    pub fn compile(self) -> Result<Shader<ts::Compiled, T, Decls>, CompilationError> {
        gl::call! {
            [propagate]
            unsafe {
                glb::CompileShader(self.object.name())
            }
        };
        self.info_log().map_or(
            // SAFETY: we just checked if shader compiled successfully
            Ok(unsafe { self.retype_to_compiled() }),
            |msg| Err(CompilationError { msg }),
        )
    }

    fn query(&self, param: QueryParam, output: &mut i32) {
        gl::call! {
            [panic]
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
            gl::call! {
                [panic]
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
            // SAFETY: todo will shader compiler should emit valid ascii?
            let message = unsafe { String::from_utf8(buffer).expect("error message is a valid utf-8") };
            message
        })
    }
}

impl<T, Decls> Shader<ts::Compiled, T, Decls>
where
    T: Target,
    Decls: uniform::bounds::Declarations,
{
    pub fn into_main(self) -> Main<T, (), (), Decls> {
        Main::new(self)
    }

    pub fn into_shared(self) -> Lib<T, Decls> {
        Lib::new(self)
    }

    pub(super) fn declarations(&self) -> uniform::Declarations<ts::Mutable, Decls> {
        self.uniform_declarations.clone()
    }
}

pub type CompiledShader<T, Decls> = Shader<ts::Compiled, T, Decls>;

pub type VertexShader<Decls = ()> = CompiledShader<target::Vertex, Decls>;
pub type TesselationControlShader<Decls = ()> = CompiledShader<target::tesselation::Control, Decls>;
pub type TesselationEvaluationShader<Decls = ()> =
    CompiledShader<target::tesselation::Evaluation, Decls>;
pub type GeometryShader<Decls = ()> = CompiledShader<target::Geometry, Decls>;
pub type FragmentShader<Decls = ()> = CompiledShader<target::Fragment, Decls>;
pub type ComputeShader<Decls = ()> = CompiledShader<target::Compute, Decls>;
