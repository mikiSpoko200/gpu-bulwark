pub mod attach;
pub mod stage;
pub mod parameters;

use frunk::labelled::chars::T;
use gl::CompileShader;

use super::prelude::Object;
use super::resource::{Allocator, self};
use super::shader::{Compiled, Shader, Main, TargetProvider, Shared, CompilationError};
use crate::gl_call;
use crate::glsl;
use crate::target::shader;
use crate::target::shader::{tesselation, Compute, Fragment, Geometry, Vertex};
use crate::types::Unimplemented;
use std::marker::PhantomData;

pub type CompiledShader<T> = Shader<T, Compiled>;

pub type VertexShader = CompiledShader<Vertex>;
pub type TesselationControlShader = CompiledShader<tesselation::Control>;
pub type TesselationEvaluationShader = CompiledShader<tesselation::Evaluation>;
pub type GeometryShader = CompiledShader<Geometry>;
pub type FragmentShader = CompiledShader<Fragment>;
pub type ComputeShader = CompiledShader<Compute>;

/// Collection of shaders for given program stage with defined stage interface.
///
/// It contains exactly one shaders that contains main function
/// and arbitrary many that are there just to supply shaders to link against.
pub(crate) struct ShaderStage<'shaders, T>
where
    T: shader::Target,
{
    pub main: &'shaders CompiledShader<T>,
    pub shared: Vec<&'shaders CompiledShader<T>>
}

impl<'s, T> ShaderStage<'s, T>
where
    T: shader::Target,
{
    pub fn new(main: &'s CompiledShader<T>) -> Self {
        Self {
            main,
            shared: Vec::new()
        }
    }
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

struct ProgramSemantics<I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub _input_phantom: PhantomData<I>,
    pub _output_phantom: PhantomData<O>,
}

impl<I: parameters::Parameters, O: parameters::Parameters> std::default::Default for ProgramSemantics<I, O> {
    fn default() -> Self {
        Self { _input_phantom: Default::default(), _output_phantom: Default::default() }
    }
}

#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    object: Object<ProgramAllocator>,
    semantics: ProgramSemantics<I, O>,
}

impl<I, O> std::default::Default for Program<I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    fn default() -> Self {
        Self { object: Default::default(), semantics: Default::default() }
    }
}

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
    ActiveUniforms = gl::ACTIVE_UNIFORMS,
    ActiveUniformBlocks = gl::ACTIVE_UNIFORM_BLOCKS,
    ActiveUniformBlockMaxNameLength = gl::ACTIVE_UNIFORM_BLOCK_MAX_NAME_LENGTH,
    ActiveUniformMaxLength = gl::ACTIVE_UNIFORM_MAX_LENGTH,
    ComputeWorkGroupSize = gl::COMPUTE_WORK_GROUP_SIZE,
    ProgramBinaryLength = gl::PROGRAM_BINARY_LENGTH,
    TransformFeedbackBufferMode = gl::TRANSFORM_FEEDBACK_BUFFER_MODE,
    TransformFeedbackVaryings = gl::TRANSFORM_FEEDBACK_VARYINGS,
    TransformFeedbackVaryingMaxLength = gl::TRANSFORM_FEEDBACK_VARYING_MAX_LENGTH,
    GeometryVerticesOut = gl::GEOMETRY_VERTICES_OUT,
    GeometryInputType = gl::GEOMETRY_INPUT_TYPE,
    GeometryOutputType = gl::GEOMETRY_OUTPUT_TYPE,
}

impl<I, O> Program<I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    // consider intoducing no input / output types so this method is not accessible
    pub fn builder<'s>(vertex_shader: &'s Main<Vertex, I, O>) -> Builder<'s, Vertex, I, O>
    where
        I: parameters::Parameters,
        O: parameters::Parameters,
    {
        Builder::new(vertex_shader)
    }
    
    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
            unsafe {
                gl::GetProgramiv(self.object.name(), param as _, output);
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

    pub(crate) fn attach<T: shader::Target>(&self, stage: &ShaderStage<T>) {
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

    pub(self) fn link(self) -> Result<Program<I, O>, CompilationError> {
        unsafe {
            gl::LinkProgram(self.object.name());
        }

        self.info_log().map_or(
            // SAFETY: we just checked if shader compiled successfully
            Ok(unsafe { self }),
            |msg| Err(CompilationError { msg }),
        )
    }
}

impl<I: parameters::Parameters, O: parameters::Parameters> resource::Bindable for Program<I, O> {
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



pub struct Builder<'shaders, T, I, O>
where
    T: shader::Target,
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    _target_phantom: PhantomData<T>,
    _input_phantom: PhantomData<I>,
    _output_phantom: PhantomData<O>,
    vertex: ShaderStage<'shaders, Vertex>,

    // todo: It would be nice to implement type state here to avoid options
    tesselation_control: Option<ShaderStage<'shaders, tesselation::Control>>,
    // Attach relation assures correctness
    tesselation_evaluation: Option<ShaderStage<'shaders, tesselation::Evaluation>>,
    geometry: Option<ShaderStage<'shaders, Geometry>>,
    fragment: Option<ShaderStage<'shaders, Fragment>>,
    compute: Option<ShaderStage<'shaders, Compute>>,
}


impl<'s, T, I, O> Builder<'s, T, I, O>
where
    T: shader::Target,
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    fn retype<NT: shader::Target, NO: parameters::Parameters>(self) -> Builder<'s, NT, I, NO> {
        Builder {
            _output_phantom: PhantomData,
            _target_phantom: PhantomData,
            _input_phantom: PhantomData,
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }
}

impl<'s, I, O> Builder<'s, Vertex, I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub fn new(vertex_shader: &'s Main<Vertex, I, O>) -> Self {
        Self {
            _target_phantom: PhantomData,
            _input_phantom: PhantomData,
            _output_phantom: PhantomData,
            vertex: ShaderStage::new(vertex_shader),
            tesselation_control: None,
            tesselation_evaluation: None,
            geometry: None,
            fragment: None,
            compute: None,
        }
    }

    pub fn vertex_shared(mut self, shader: &'s Shared<Vertex>) -> Self {    
        self.vertex.shared.push(shader);
        self
    }

    pub fn tesselation_control_main<TCO: parameters::Parameters>(mut self, shader: &'s Main<tesselation::Control, O, TCO>) -> Builder<tesselation::Control, I, TCO> {    
        self.tesselation_control = Some(ShaderStage::new(shader));
        self.retype()
    }

    pub fn tesselation_evaluation_main<TEO: parameters::Parameters>(mut self, shader: &'s Main<tesselation::Evaluation, O, TEO>) -> Builder<tesselation::Control, I, TEO> {    
        self.tesselation_evaluation = Some(ShaderStage::new(shader));
        self.retype()
    }

    pub fn geometry_main<GO: parameters::Parameters>(mut self, geometry: &'s Main<Geometry, O, GO>) -> Builder<Geometry, I, GO> {
        self.geometry = Some(ShaderStage::new(geometry));
        self.retype()
    }

    pub fn fragment_main<FO: parameters::Parameters>(mut self, fragment: &'s Main<Fragment, O, FO>) -> Builder<Fragment, I, FO> {
        self.fragment = Some(ShaderStage::new(fragment));
        self.retype()
    }
}

impl<'s, I, O> Builder<'s, tesselation::Control, I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub fn tesselation_control_shared(mut self, shared: &'s Shared<tesselation::Control>) -> Self {
        self.tesselation_control.as_mut().expect("tesselation control was initialized").shared.push(shared);
        self
    }

    pub fn tesselation_evaluation_main<TEO>(mut self, shader: &'s Main<tesselation::Evaluation, O, TEO>) -> Builder<tesselation::Evaluation, I, TEO>
    where
        TEO: parameters::Parameters,
    {    
        self.tesselation_evaluation = Some(ShaderStage::new(shader));
        self.retype()
    }
}

impl<'s, I, O> Builder<'s, tesselation::Evaluation, I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub fn tesselation_evaluation_shared(mut self, shared: &'s Shared<tesselation::Evaluation>) -> Self {
        self.tesselation_evaluation.as_mut().expect("tesselation evaluation stage was initialized").shared.push(shared);
        self
    }

    pub fn geometry_main<GO: parameters::Parameters>(mut self, geometry: &'s Main<Geometry, O, GO>) -> Builder<Geometry, I, GO> {
        self.geometry = Some(ShaderStage::new(geometry));
        self.retype()
    }

    pub fn fragment_main<FO: parameters::Parameters>(mut self, fragment: &'s Main<Fragment, O, FO>) -> Builder<Fragment, I, FO> {
        self.fragment = Some(ShaderStage::new(fragment));
        self.retype()
    }
}

impl<'s, I, O> Builder<'s, Geometry, I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub fn geometry_shared(mut self, shared: &'s Shared<Geometry>) -> Self {
        self.geometry.as_mut().expect("geometry stage was initialized").shared.push(shared);
        self
    }

    pub fn fragment_main<FO: parameters::Parameters>(mut self, fragment: &'s Main<Fragment, O, FO>) -> Builder<Fragment, I, FO> {
        self.fragment = Some(ShaderStage::new(fragment));
        self.retype()
    }
}

impl<'s, I, O> Builder<'s, Fragment, I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    pub fn fragment_shared(mut self, shared: &'s Shared<Fragment>) -> Self {
        self.fragment.as_mut().expect("fragment stage was initialized").shared.push(shared);
        self
    }

    pub fn build(&self) -> Result<Program<I, O>, CompilationError> {
        let program = Program::default();
        program.attach(&self.vertex);

        if let (Some(control_stage), Some(evaluation_stage)) = (&self.tesselation_control, &self.tesselation_evaluation) {
            program.attach(control_stage);
            program.attach(evaluation_stage);
        }

        if let Some(geometry) = &self.geometry {
            program.attach(geometry);
        }

        if let Some(fragment) = &self.geometry {
            program.attach(fragment);
        }

        program.link()
    }
}

impl<'s, I, O> Builder<'s, Vertex, I, O>
where
    I: parameters::Parameters,
    O: parameters::Parameters,
{
    
} 
