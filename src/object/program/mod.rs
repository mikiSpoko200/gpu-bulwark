pub mod stage;
pub mod uniform;
pub mod builder;
pub(super) mod internal;

use std::marker::PhantomData;

use frunk::labelled::chars::T;
use gl;
use glutin::error;

use uniform::Definitions;
pub use builder::Builder;

use self::uniform::Definition;

use super::shader;
pub(self) use super::shader::prelude::*;
use super::shader::{parameters, parameters::Parameters};


use super::prelude::Object;
use super::resource::{Allocator, self};
use crate::glsl::location::{Validated, Location};
use crate::{gl_call, hlist};
use crate::glsl;
use crate::hlist::indexed;

#[repr(u32)]
pub enum QueryParam {
    DeleteStatus = gl::DELETE_STATUS,
    LinkStatus = gl::LINK_STATUS,
    ValidateStatus = gl::VALIDATE_STATUS,
    InfoLogLength = gl::INFO_LOG_LENGTH,
    AttachedShaders = gl::ATTACHED_SHADERS,
    ActiveAtomicCounterBuffers = gl::ACTIVE_ATOMIC_COUNTER_BUFFERS,
    ActiveAttributes = gl::ACTIVE_ATTRIBUTES,
    ActiveAttributeMaxLength = gl::ACTIVE_ATTRIBUTE_MAX_LENGTH,
    Activeuniforms = gl::ACTIVE_UNIFORMS,
    ActiveuniformBlocks = gl::ACTIVE_UNIFORM_BLOCKS,
    ActiveuniformBlockMaxNameLength = gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
    ActiveuniformMaxLength = gl::ACTIVE_UNIFORM_MAX_LENGTH,
    ComputeWorkGroupSize = gl::COMPUTE_WORK_GROUP_SIZE,
    ProgramBinaryLength = gl::PROGRAM_BINARY_LENGTH,
    TransformFeedbackBufferMode = gl::TRANSFORM_FEEDBACK_BUFFER_MODE,
    TransformFeedbackVaryings = gl::TRANSFORM_FEEDBACK_VARYINGS,
    TransformFeedbackVaryingMaxLength = gl::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH,
    GeometryVerticesOut = gl::GEOMETRY_VERTICES_OUT,
    GeometryInputType = gl::GEOMETRY_INPUT_TYPE,
    GeometryOutputType = gl::GEOMETRY_OUTPUT_TYPE,
}

#[derive(thiserror::Error, Debug)]
#[error("program linking failed {msg}")]
pub struct LinkingError {
    msg: String
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
            *name = unsafe { gl::CreateProgram() };
            // TODO: Check for errors
        }
    }

    fn free(names: &[u32]) {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for &name in names {
            unsafe { gl::DeleteProgram(name) };
            // TODO: Check for errors
        }
    }
}

struct ProgramPhantomData<I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub _input_phantom: PhantomData<I>,
    pub _output_phantom: PhantomData<O>,
}

impl<I: parameters::Parameters, O: parameters::Parameters> std::default::Default for ProgramPhantomData<I, O> {
    fn default() -> Self {
        Self { _input_phantom: Default::default(), _output_phantom: Default::default() }
    }
}

#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<IS, OS, DUS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: uniform::marker::Definitions
{
    object: Object<ProgramAllocator>,
    _phantoms: ProgramPhantomData<IS, OS>,
    defined_uniforms: uniform::Definitions<DUS>,
}

impl Program<(), (), ()> {
    pub fn builder<'s>() -> Builder<'s, Vertex, (), (), (), ()> {
        Builder::new()
    }
}

impl<IS, OS> Program<IS, OS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    pub fn create() -> Self {
        Self {
            object: Default::default(),
            _phantoms: Default::default(),
            defined_uniforms: Definitions::default(),
        }
    }

    pub fn create_with_uniforms<DUS: uniform::marker::Definitions>(uniforms: uniform::Uniforms<DUS, ()>) -> Program<IS, OS, DUS> {
        Program {
            object: Default::default(),
            _phantoms: Default::default(),
            defined_uniforms: uniforms.definitions,
        }
    }
}

impl<IS, OS> std::default::Default for Program<IS, OS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    fn default() -> Self {
        Self::create()
    }
}

impl<IS, OS, DUS> Program<IS, OS, DUS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: uniform::marker::Definitions
{
    // pub fn add_uniform_definition<GLU, GLSLU, const LOCATION: usize>(self, uniform: GLU, location: &Location<GLSLU, LOCATION, Validated>) -> Program<IS, OS, (DUS, Definition<GLU, GLSLU, LOCATION>)>
    // where
    //     GLSLU: glsl::Uniform,
    //     GLU: Clone,
    //     (GLU, GLSLU): glsl::compatible::Compatible<GLU, GLSLU>
    
    // {
    //     let extended = self.defined_uniforms.define(uniform, location);
    //     Program {
    //         object: self.object,
    //         _phantoms: self._phantoms,
    //         defined_uniforms: extended,
    //     }
    // }

    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
            unsafe {
                gl::GetProgramiv(self.object.name(), param as _, output);
            }
        }
    }

    pub fn info_log(&self) -> Option<String> {
        let mut successful = 0;
        self.query(QueryParam::LinkStatus, &mut successful);

        if successful == gl::TRUE as _ {
            return None;
        }

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
                    gl::GetProgramInfoLog(
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
                gl::AttachShader(self.object.name(), main.object.name());
            }
        }
        for shared in &stage.shared {
            gl_call! {
                #[panic]
                unsafe {
                    gl::AttachShader(self.object.name(), shared.object.name());
                }
            }
        }
    }

    pub(self) fn link(self) -> Result<Program<IS, OS, DUS>, LinkingError> {
        unsafe {
            gl::LinkProgram(self.object.name());
        }

        self.info_log().map_or(
            // SAFETY: we just checked if shader compiled successfully
            Ok(unsafe { self }),
            |msg| Err(LinkingError { msg }),
        )
    }
}

impl<IS, OS, DUS> resource::Bindable for Program<IS, OS, DUS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: uniform::marker::Definitions
{
    fn bind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                gl::UseProgram(self.object.name())
            }
        }
    }

    fn unbind(&self) {
        gl_call! {
            #[panic]
            unsafe {
                // todo: should this be the case?
                gl::UseProgram(0)
            }
        }
    }
}
