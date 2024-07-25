pub mod builder;
pub(super) mod internal;
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
use glsl::binding;
use glsl::prelude::*;

use binding::marker::{layout, storage};
use glutin::error;

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

enum ProgramAllocator {}

unsafe impl Allocator for ProgramAllocator {
    fn allocate(names: &mut [u32]) {
        for name in names {
            *name = unsafe { glb::CreateProgram() };
            // TODO: Check for errors
        }
    }

    fn free(names: &[u32]) {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for &name in names {
            unsafe { glb::DeleteProgram(name) };
            // TODO: Check for errors
        }
    }
}

pub enum ProgramBinder {}

impl Binder for ProgramBinder {
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

#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<Ins, Outs, Decls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    object: ObjectBase<Self>,
    state: ProgramState<Ins, Outs, Decls>,
}

impl<Ins, Outs, Decls> std::ops::Deref for Program<Ins, Outs, Decls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Decls: uniform::bounds::Declarations,
{
    type Target = ObjectBase<Self>;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<Ins, Outs, Decls> Object for Program<Ins, Outs, Decls>
where
    Ins: glsl::parameters::Parameters<glsl::binding::marker::storage::In>,
    Outs: glsl::parameters::Parameters<glsl::binding::marker::storage::Out>,
    Decls: gl::uniform::bounds::Declarations,
{
    type Binder = ProgramBinder;
    type Allocator = ProgramAllocator;
}

impl Program<(), (), ()> {
    pub fn builder<'s>() -> Builder<'s, Vertex, (), (), (), ()> {
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
            state: Default::default(),
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

    fn attach<T: shader::target::Target>(&self, stage: &internal::ShaderStage<T>) {
        let main = stage.main;
        gl::call! {
            [panic]
            unsafe {
                glb::AttachShader(self.object.name(), main.object.name());
            }
        }
        for shared in &stage.libs {
            gl::call! {
                [panic]
                unsafe {
                    glb::AttachShader(self.object.name(), shared.object.name());
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

    /// Set new value for given uniform binding
    pub fn uniform<GLSL, const LOCATION: usize, IDX>(
        &mut self,
        binding: &UniformBinding<GLSL, LOCATION>,
        uniform: impl glsl::Compatible<GLSL>,
    ) where
        GLSL: glsl::bounds::TransparentUniform,
        IDX: Index,
        DUS: Find<UniformBinding<GLSL, LOCATION>, IDX>,
    {
        self.bound(|_| GLSL::set(binding, uniform));
    }
}

impl<IS, OS, DUS> Binder for Program<IS, OS, DUS>
where
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    DUS: uniform::bounds::Declarations,
{
    fn bind(&self) {
        gl::call! {
            [panic]
            unsafe {
                glb::UseProgram(self.object.name())
            }
        }
    }

    fn unbind(&self) {
        gl::call! {
            [panic]
            unsafe {
                glb::UseProgram(0)
            }
        }
    }
}
