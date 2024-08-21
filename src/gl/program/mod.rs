pub mod builder;
pub mod stage;

use crate::prelude::internal::*;

use crate::ts;

pub use builder::Builder;

use crate::gl;
use crate::glsl;
use crate::hlist;
use crate::hlist::counters::Index;
use crate::hlist::indexed;
use crate::hlist::lhlist::Find;
use crate::valid;

use gl::object::*;
use gl::shader;
use gl::shader::prelude::*;
use gl::uniform;
use glsl::variable;

use variable::UniformVariable;

use variable::{layout, storage};

use super::uniform::Declaration;
use super::uniform::Declarations;
use super::uniform::Definitions;

/// Collection of shaders for given program stage with defined stage interface.
///
/// It contains exactly one shaders that contains main function
/// and arbitrary many that are there just to supply shaders to link against.
pub(in crate::gl) struct ShaderStage<'shaders, T>
where
    T: shader::target::Target,
{
    pub main: &'shaders ObjectBase<shader::ShaderObject<T>>,
    pub libs: Vec<&'shaders ObjectBase<shader::ShaderObject<T>>>,
}

impl<'s, T> ShaderStage<'s, T>
where
    T: shader::target::Target,
{
    pub fn new<Decls>(main: &'s shader::Shader<ts::Compiled, T, Decls>) -> ShaderStage<'s, T>
    where 
        Decls: uniform::bounds::Declarations,
    {
        ShaderStage {
            main: main,
            libs: Vec::new(),
        }
    }
}

#[repr(u32)]
pub enum QueryParam {
    DeleteStatus = glb::DELETE_STATUS,
    LinkStatus = glb::LINK_STATUS,
    ValidateStatus = glb::VALIDATE_STATUS,
    InfoLogLength = glb::INFO_LOG_LENGTH,
    AttachedShaders = glb::ATTACHED_SHADERS,
    ActiveAtomicCounterBuffers = glb::ACTIVE_ATOMIC_COUNTER_BUFFERS,
    ActiveAttributes = glb::ACTIVE_ATTRIBUTES,
    ActiveAttributeMaxLength = glb::ACTIVE_ATTRIBUTE_MAX_LENGTH,
    ActiveUniforms = glb::ACTIVE_UNIFORMS,
    ActiveUniformBlocks = glb::ACTIVE_UNIFORM_BLOCKS,
    ActiveUniformBlockMaxNameLength = glb::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
    ActiveUniformMaxLength = glb::ACTIVE_UNIFORM_MAX_LENGTH,
    ComputeWorkGroupSize = glb::COMPUTE_WORK_GROUP_SIZE,
    ProgramBinaryLength = glb::PROGRAM_BINARY_LENGTH,
    TransformFeedbackBufferMode = glb::TRANSFORM_FEEDBACK_BUFFER_MODE,
    TransformFeedbackVaryings = glb::TRANSFORM_FEEDBACK_VARYINGS,
    TransformFeedbackVaryingMaxLength = glb::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH,
    GeometryVerticesOut = glb::GEOMETRY_VERTICES_OUT,
    GeometryInputType = glb::GEOMETRY_INPUT_TYPE,
    GeometryOutputType = glb::GEOMETRY_OUTPUT_TYPE,
}

#[derive(thiserror::Error, Debug)]
#[error("program linking failed {msg}")]
pub struct LinkingError {
    msg: String,
}

pub trait LinkingStatus {}

pub struct UnLinked;
impl LinkingStatus for UnLinked {}

pub struct Linked;
impl LinkingStatus for Linked {}

#[hi::mark(PartialObject, Object)]
pub enum ProgramObject { }

unsafe impl Allocator for ProgramObject {
    fn allocate(names: &mut [u32]) {
        for name in names {
            *name = unsafe { glb::CreateProgram() };
            // TODO: Check for errors
        }
    }

    fn free(names: &[u32]) {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for &name in names {
            // TODO: Check for errors
            gl::call! {
                [panic]
                unsafe {
                    glb::DeleteProgram(name)
                }
            }
        }
    }
}

impl Binder for ProgramObject {
    fn bind(name: u32) {
        gl::call! {
            [panic]
            unsafe {
                glb::UseProgram(name);
            }
        }
    }
}

struct ProgramState<Ins, Outs, Decls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    pub _phantoms: PhantomData<(Ins, Outs)>,
    pub uniform_declarations: uniform::Declarations<ts::Immutable, Decls>,
}

impl<Ins, Outs> Default for ProgramState<Ins, Outs, ()>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
{
    fn default() -> Self {
        Self {
            _phantoms: Default::default(),
            uniform_declarations: uniform::Declarations::<ts::Immutable, _>::default(),
        }
    }
}

impl<Ins, Outs, Decls> ProgramState<Ins, Outs, Decls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    pub fn new(decls: uniform::Declarations<ts::Mutable, Decls>) -> Self {
        Self {
            _phantoms: PhantomData,
            uniform_declarations: decls.into_immutable(),
        }
    }
}

#[derive(dm::Deref)]
#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<Ins, Outs, Decls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    #[deref]
    object: ObjectBase<ProgramObject>,
    state: ProgramState<Ins, Outs, Decls>,
}

impl Program<(), (), ()> {
    pub fn builder<'s>() -> Builder<'s, ts::None, (), (), (), ()> {
        Builder::new()
    }
}

impl<Ins, Outs> Program<Ins, Outs, ()>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
{
    pub fn create() -> Self {
        Self {
            object: Default::default(),
            state: Default::default(),
        }
    }

    pub fn create_with_uniforms<Defs>(definitions: &uniform::Definitions<Defs>) -> Program<Ins, Outs, Defs::AsDeclarations>
    where
        Defs: uniform::bounds::Definitions,
    {
        Program {
            object: Default::default(),
            state: ProgramState::new(Declarations(PhantomData)),
        }
    }
}

impl<IS, OS, DUS> Program<IS, OS, DUS>
where
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    DUS: uniform::bounds::Declarations,
{
    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl::call! {
            [panic]
            unsafe {
                glb::GetProgramiv(self.object.name(), param as _, output);
            }
        }
    }

    pub fn info_log(&self) -> Option<String> {
        let mut successful = 0;
        self.query(QueryParam::LinkStatus, &mut successful);

        if successful == glb::TRUE as _ {
            return None;
        }

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
                    glb::GetProgramInfoLog(
                        self.object.name(),
                        buffer.capacity() as _,
                        &mut actual_length,
                        buffer.as_mut_ptr() as _
                    )
                }
            }
            // GetShaderInfoLog does not account for null terminator in returned length.
            // SAFETY: nothing will panic here so it's safe to set length.
            unsafe {
                buffer.set_len((actual_length) as _);
            }
            // SAFETY: todo will shader compiler should emit valid ascii?
            unsafe { String::from_utf8_unchecked(buffer) }
        })
    }

    fn attach<T: shader::target::Target>(&self, stage: &ShaderStage<T>) {
        let main = stage.main;
        gl::call! {
            [panic]
            unsafe {
                glb::AttachShader(self.object.name(), main.name());
            }
        }
        for lib in &stage.libs {
            gl::call! {
                [panic]
                unsafe {
                    glb::AttachShader(self.object.name(), lib.name());
                }
            }
        }
    }

    pub(self) fn link(self) -> Result<Program<IS, OS, DUS>, LinkingError> {
        unsafe {
            glb::LinkProgram(self.object.name());
        }

        self.info_log().map_or(
            // SAFETY: we just checked if shader compiled successfully
            Ok(unsafe { self }),
            |msg| Err(LinkingError { msg }),
        )
    }

    /// Set new value for given uniform variable
    pub fn uniform<GLSL, const LOCATION: usize, IDX>(
        &mut self,
        var: &UniformVariable<GLSL, LOCATION>,
        uniform: &impl glsl::Compatible<GLSL>,
    ) where
        GLSL: glsl::bounds::TransparentUniform,
        IDX: Index,
        DUS: Find<UniformVariable<GLSL, LOCATION>, IDX>,
    {
        self.bound(|_| GLSL::set(var, uniform));
    }
}
