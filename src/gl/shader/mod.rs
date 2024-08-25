use crate::hlist::lhlist::Concatenate;
use crate::prelude::internal::*;

pub mod lib;
pub mod main;
pub mod prelude;
pub mod target;

use crate::gl;
use crate::glsl;
use crate::ts;
use crate::hlist::lhlist;
use crate::hlist::indexed::rhlist;
use gl::uniform;

// Reexports
pub use target::Target;
pub(super) use lib::Lib;
pub(super) use main::Main;

use gl::object::{ObjectBase, PartialObject};
use glsl::TransparentUniformVariable;

use super::uniform::Declarations;

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

/// Allocator for Shader Objects.
#[hi::mark(PartialObject)]
pub struct ShaderObject<T>(PhantomData<T>)
where
    T: Target;

unsafe impl<T: Target> gl::object::Allocator for ShaderObject<T> {
    fn allocate(names: &mut [u32]) {
        for name in names {
            gl::call! {
                [panic]
                *name = unsafe { glb::CreateShader(T::ID) }
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

pub(in crate::gl) struct ShaderPhantom<T, C>(PhantomData<(T, C)>)
where
    T: Target,
    C: ts::Compilation,
;

impl<T, C> Default for ShaderPhantom<T, C>
where
    T: Target,
    C: ts::Compilation,
{
    fn default() -> Self {
        Self(PhantomData)
    }
}

pub fn create<T: Target>() -> Shader<ts::Uncompiled, T, ()> {
    Shader::create()
}

#[derive(dm::Deref)]
pub struct Shader<C, T, Decls>
where
    C: ts::Compilation,
    T: Target,
    Decls: uniform::bounds::Declarations,
{
    #[deref]
    pub(in crate::gl) object: ObjectBase<ShaderObject<T>>,
    pub(in crate::gl) state: ShaderPhantom<T, C>,
    pub(in crate::gl) uniform_declarations: uniform::Declarations<ts::Mutable, Decls>,
}

impl<T: Target> Default for Shader<ts::Uncompiled, T, ()> {
    fn default() -> Self {
        Self::create()
    }
}

impl<T: Target> Shader<ts::Uncompiled, T, ()> {
    pub fn create() -> Self {
        Self {
            object: ObjectBase::default(),
            state: ShaderPhantom::default(),
            uniform_declarations: uniform::Declarations::default(),
        }
    }
}

// TODO: common functionality for uniform setting 
impl<T, Decls> Shader<ts::Uncompiled, T, Decls>
where
    T: Target,
    Decls: uniform::bounds::Declarations,
{ 
    /// Declare uniform variable used by this shader
    pub fn uniform<U, const LOCATION: usize>(self, _: &TransparentUniformVariable<U, LOCATION>) -> Shader<ts::Uncompiled, T, (Decls, glsl::TransparentUniformVariable<U, LOCATION>)>
    where
        U: glsl::Uniform,
    {
        Shader {
            uniform_declarations: uniform::Declarations::default(),
            object: self.object,
            state: self.state,
        }
    }

    /// Declare uniform variable used by this shader
    pub fn uniforms<Unis, NDecls>(self, _: &Unis) -> Shader<ts::Uncompiled, T, Decls::Concatenated>
    where
        Unis: glsl::TransparentUniforms + Into<NDecls>,
        Decls: Concatenate<NDecls, Concatenated: uniform::bounds::Declarations>,
    {
        Shader {
            uniform_declarations: uniform::Declarations::default(),
            object: self.object,
            state: self.state,
        }
    }


    /// Add source for shader.
    pub fn source(&mut self, sources: &[&str]) -> &Self {
        let pointers: Vec<_> = sources.iter().map(|s| s.as_ptr()).collect();
        let lengths: Vec<_> = sources.iter().map(|s| s.len()).collect();

        gl::call! {
            [panic]
            unsafe {
                glb::ShaderSource(
                    self.name(),
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
            state: ShaderPhantom::default(),
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
            let mut buffer = Vec::<u8>::with_capacity(log_size as _);
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
pub type TesselationEvaluationShader<Decls = ()> = CompiledShader<target::tesselation::Evaluation, Decls>;
pub type GeometryShader<Decls = ()> = CompiledShader<target::Geometry, Decls>;
pub type FragmentShader<Decls = ()> = CompiledShader<target::Fragment, Decls>;
pub type ComputeShader<Decls = ()> = CompiledShader<target::Compute, Decls>;
