pub mod builder;
pub(super) mod internal;
pub mod stage;
pub mod uniform;

use std::marker::PhantomData;

use frunk::labelled::chars::T;
use glb;
use glutin::error;

use super::shader;
pub use builder::Builder;

pub(self) use super::shader::prelude::*;

use super::prelude::Object;
use super::resource::{self, Allocator};
use crate::glsl::{self, binding};
use crate::hlist::counters::Index;
use crate::hlist::indexed;
use crate::hlist::lhlist::Find;
use crate::{gl_call, hlist};

use crate::glsl::prelude::*;

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
    Activeuniforms = glb::ACTIVE_UNIFORMS,
    ActiveuniformBlocks = glb::ACTIVE_UNIFORM_BLOCKS,
    ActiveuniformBlockMaxNameLength = glb::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
    ActiveuniformMaxLength = glb::ACTIVE_UNIFORM_MAX_LENGTH,
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

#[derive(Default)]
struct ProgramAllocator;

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

use binding::marker::{storage, layout};

struct ProgramPhantomData<I, O>
where
    I: glsl::Parameters<storage::In>,
    O: glsl::Parameters<storage::Out>,
{
    pub _input_phantom: PhantomData<I>,
    pub _output_phantom: PhantomData<O>,
}

impl<I, O> std::default::Default for ProgramPhantomData<I, O>
where
    I: glsl::Parameters<storage::In>,
    O: glsl::Parameters<storage::Out>,
{
    fn default() -> Self {
        Self {
            _input_phantom: Default::default(),
            _output_phantom: Default::default(),
        }
    }
}

#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<IS, OS, DUS>
where
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    DUS: glsl::Uniforms,
{
    object: Object<ProgramAllocator>,
    _phantoms: ProgramPhantomData<IS, OS>,
    defined_uniforms: DUS,
}

impl Program<(), (), ()> {
    pub fn builder<'s>() -> Builder<'s, Vertex, (), (), (), ()> {
        Builder::new()
    }
}

impl<IS, OS> Program<IS, OS, ()>
where
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
{
    pub fn create() -> Self {
        Self {
            object: Default::default(),
            _phantoms: Default::default(),
            defined_uniforms: (),
        }
    }

    pub fn create_with_uniforms<DUS>(uniforms: uniform::Matcher<DUS, ()>) -> Program<IS, OS, DUS>
    where
        DUS: uniform::marker::Definitions
    {
        Program {
            object: Default::default(),
            _phantoms: Default::default(),
            defined_uniforms: uniforms.definitions,
        }
    }
}

impl<IS, OS, DUS> Program<IS, OS, DUS>
where
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    DUS: uniform::marker::Definitions,
{
    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
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
            gl_call! {
                #[panic]
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
            // SAFETY: todo will shader compiler should emmit valid ascii?
            unsafe { String::from_utf8_unchecked(buffer) }
        })
    }

    fn attach<T: shader::target::Target>(&self, stage: &internal::ShaderStage<T>) {
        let main = stage.main;
        gl_call! {
            #[panic]
            unsafe {
                glb::AttachShader(self.object.name(), main.object.name());
            }
        }
        for shared in &stage.shared {
            gl_call! {
                #[panic]
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

    pub fn uniform<GLU, GLSLU, const LOCATION: usize, IDX>(
        &mut self,
        binding: &UniformBinding<GLSLU, LOCATION>,
        uniform: impl glsl::compatible::Compatible<GLSLU>,
    ) where
        GLSLU: glsl::Uniform<Group = glsl::marker::Transparent> + glsl::uniform::ops::Set,
        IDX: Index,
        DUS: Find<UniformDefinition<GLSLU, LOCATION>, IDX>,
    {
        use crate::gl::resource::Bind;

        self.bind();
        self.defined_uniforms.uniform(binding, uniform);
    }
}

impl<IS, OS, DUS> resource::Bind for Program<IS, OS, DUS>
where
    IS: glsl::Parameters<storage::In>,
    OS: glsl::Parameters<storage::Out>,
    DUS: uniform::marker::Definitions,
{
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                glb::UseProgram(self.object.name())
            }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                // todo: should this be the case?
                glb::UseProgram(0)
            }
        }
    }
}
