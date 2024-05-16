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

use super::prelude::Object;
use super::resource::{Allocator, self};
use crate::hlist::counters::Index;
use crate::hlist::lhlist::Find;
use crate::{gl_call, hlist};
use crate::glsl::{self, binding};
use crate::hlist::indexed;

use crate::glsl::prelude::*;

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
    I: glsl::Parameters<In>,
    O: glsl::Parameters<Out>,
{
    pub _input_phantom: PhantomData<I>,
    pub _output_phantom: PhantomData<O>,
}

impl<I: glsl::Parameters<In>, O: glsl::Parameters<Out>> std::default::Default for ProgramPhantomData<I, O> {
    fn default() -> Self {
        Self { _input_phantom: Default::default(), _output_phantom: Default::default() }
    }
}

#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<IS, OS, DUS>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
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
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
{
    pub fn create() -> Self {
        Self {
            object: Default::default(),
            _phantoms: Default::default(),
            defined_uniforms: Definitions::default(),
        }
    }

    pub fn create_with_uniforms<DUS: glsl::Uniforms>(uniforms: uniform::UniformsBuilder<DUS, ()>) -> Program<IS, OS, DUS> {
        Program {
            object: Default::default(),
            _phantoms: Default::default(),
            defined_uniforms: uniforms.definitions,
        }
    }
}

impl<IS, OS, DUS> Program<IS, OS, DUS>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
    DUS: uniform::marker::Definitions
{
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
            let mut buffer = Vec::<u8>::with_capacity(log_size as _);
            let mut actual_length = 0;
            gl_call! {
                #[panic]
                // SAFETY: All values passed are valid
                // todo: notes on error situations
                unsafe {
                    gl::GetProgramInfoLog(
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

    pub fn uniform<GLU, GLSLU, const LOCATION: usize, IDX>(&mut self, binding: &UniformBinding<GLSLU, LOCATION>, uniform: &GLU)
    where
        GLSLU: glsl::Uniform<Primitive = <GLU as glsl::FFI>::Primitive> + glsl::uniform::ops::Set,
        GLU: glsl::compatible::Compatible<GLSLU>, // Primitive = GLU::Primitive
        IDX: Index,
        DUS: Find<Definition<GLU, GLSLU, LOCATION>, IDX>,
    {
        use crate::object::resource::Bind;

        self.bind();
        self.defined_uniforms.uniform(binding, uniform);
    }

    pub fn uniform_f<const SIZE: usize, const LOCATION: usize, IDX>(&mut self, binding: &UniformBinding<glsl::base::Vec<f32, SIZE>, LOCATION>, uniform: &[f32; SIZE])
    where
        glsl::base::Vec<f32, SIZE>: glsl::Uniform<Primitive = <[f32; SIZE] as glsl::FFI>::Primitive> + glsl::uniform::ops::Set,
        [f32; SIZE]: glsl::compatible::Compatible<glsl::base::Vec<f32, SIZE>>,
        glsl::Const<SIZE>: glsl::marker::VecSize,
        IDX: Index,
        DUS: Find<Definition<[f32; SIZE], glsl::base::Vec<f32, SIZE>, LOCATION>, IDX>,
    {
        self.uniform(binding, uniform);
    }

    pub fn uniform_i<const SIZE: usize, const LOCATION: usize, IDX>(&mut self, binding: &UniformBinding<glsl::base::Vec<i32, SIZE>, LOCATION>, uniform: &[i32; SIZE])
    where
        glsl::base::Vec<i32, SIZE>: glsl::Uniform<Primitive = <[i32; SIZE] as glsl::FFI>::Primitive> + glsl::uniform::ops::Set,
        [i32; SIZE]: glsl::compatible::Compatible<glsl::base::Vec<i32, SIZE>>,
        glsl::Const<SIZE>: glsl::marker::VecSize,
        IDX: Index,
        DUS: Find<Definition<[i32; SIZE], glsl::base::Vec<i32, SIZE>, LOCATION>, IDX>,
    {
        self.uniform(binding, uniform);
    }

    pub fn uniform_ui<const SIZE: usize, const LOCATION: usize, IDX>(&mut self, binding: &UniformBinding<glsl::base::Vec<u32, SIZE>, LOCATION>, uniform: &[u32; SIZE])
    where
        glsl::base::Vec<u32, SIZE>: glsl::Uniform<Primitive = <[u32; SIZE] as glsl::FFI>::Primitive> + glsl::uniform::ops::Set,
        [u32; SIZE]: glsl::compatible::Compatible<glsl::base::Vec<u32, SIZE>>,
        glsl::Const<SIZE>: glsl::marker::VecSize,
        IDX: Index,
        DUS: Find<Definition<[u32; SIZE], glsl::base::Vec<u32, SIZE>, LOCATION>, IDX>,
    {
        self.uniform(binding, uniform);
    }

    pub fn uniform_1f<const LOCATION: usize>(&mut self, _: &UniformBinding<f32, LOCATION>, uniform: f32) {
        
    }
    pub fn uniform_2f<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Vec2, LOCATION>, uniform: &[f32; 2]) {

    }
    pub fn uniform_3f<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Vec3, LOCATION>, uniform: &[f32; 3]) {

    }
    pub fn uniform_4f<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Vec4, LOCATION>, uniform: &[f32; 4]) {

    }

    pub fn uniform_1i<const LOCATION: usize, IDX>(&mut self, binding: &UniformBinding<i32, LOCATION>, uniform: i32) where IDX: Index, DUS: Find<Definition<i32, i32, LOCATION>, IDX> {
        self.uniform(binding, &uniform);
    }
    pub fn uniform_2i<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::IVec2, LOCATION>, uniform: &[i32; 2]) {

    }
    pub fn uniform_3i<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::IVec3, LOCATION>, uniform: &[i32; 3]) {

    }
    pub fn uniform_4i<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::IVec4, LOCATION>, uniform: &[i32; 4]) {

    }
    
    pub fn uniform_1ui<const LOCATION: usize>(&mut self, _: &UniformBinding<u32, LOCATION>, uniform: u32) {

    }
    pub fn uniform_2ui<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::UVec2, LOCATION>, uniform: &[u32; 2]) {

    }
    pub fn uniform_3ui<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::UVec3, LOCATION>, uniform: &[u32; 3]) {

    }
    pub fn uniform_4ui<const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::UVec4, LOCATION>, uniform: &[u32; 4]) {

    }

    pub fn uniform_1fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<f32, N>, LOCATION>, value: &[f32; N]) {

    }
    pub fn uniform_2fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Vec2, N>, LOCATION>, value: &[[f32; 2]; N]) {

    }
    pub fn uniform_3fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Vec3, N>, LOCATION>, value: &[[f32; 3]; N]) {

    }
    pub fn uniform_4fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Vec4, N>, LOCATION>, value: &[[f32; 4]; N]) {

    }

    pub fn uniform_1iv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<i32        , N>, LOCATION>, value: &[i32     ; N]) { }
    pub fn uniform_2iv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::IVec2, N>, LOCATION>, value: &[[i32; 2]; N]) { }
    pub fn uniform_3iv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::IVec3, N>, LOCATION>, value: &[[i32; 3]; N]) { }
    pub fn uniform_4iv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::IVec4, N>, LOCATION>, value: &[[i32; 4]; N]) { }

    pub fn uniform_1uiv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<u32        , N>, LOCATION>, value: &[u32     ; N]) { }
    pub fn uniform_2uiv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::UVec2, N>, LOCATION>, value: &[[u32; 2]; N]) { }
    pub fn uniform_3uiv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::UVec3, N>, LOCATION>, value: &[[u32; 3]; N]) { }
    pub fn uniform_4uiv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::UVec4, N>, LOCATION>, value: &[[u32; 4]; N]) { }

    pub fn uniform_matrix_2fv  <const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat2  , N>, LOCATION>, transpose: bool, value: &[[f32; 2 * 2]; N]) { }
    pub fn uniform_matrix_3fv  <const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat3  , N>, LOCATION>, transpose: bool, value: &[[f32; 3 * 3]; N]) { }
    pub fn uniform_matrix_4fv  <const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat4  , N>, LOCATION>, transpose: bool, value: &[[f32; 4 * 4]; N]) { }
    pub fn uniform_matrix_2x3fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat2x3, N>, LOCATION>, transpose: bool, value: &[[f32; 2 * 3]; N]) { }
    pub fn uniform_matrix_2x4fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat2x4, N>, LOCATION>, transpose: bool, value: &[[f32; 2 * 4]; N]) { }
    pub fn uniform_matrix_3x2fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat3x2, N>, LOCATION>, transpose: bool, value: &[[f32; 3 * 2]; N]) { }
    pub fn uniform_matrix_3x4fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat3x4, N>, LOCATION>, transpose: bool, value: &[[f32; 3 * 4]; N]) { }
    pub fn uniform_matrix_4x2fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat4x2, N>, LOCATION>, transpose: bool, value: &[[f32; 4 * 2]; N]) { }
    pub fn uniform_matrix_4x3fv<const N: usize, const LOCATION: usize>(&mut self, _: &UniformBinding<glsl::Array<glsl::Mat4x3, N>, LOCATION>, transpose: bool, value: &[[f32; 4 * 3]; N]) { }
}

impl<IS, OS, DUS> resource::Bind for Program<IS, OS, DUS>
where
    IS: glsl::Parameters<In>,
    OS: glsl::Parameters<Out>,
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
