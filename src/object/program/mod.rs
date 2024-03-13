pub mod attach;
pub mod stage;
pub mod parameters;
pub mod uniform;

use frunk::labelled::chars::T;
use gl;
use glutin::error;

use self::uniform::Index;

use super::prelude::Object;
use super::resource::{Allocator, self};
use super::shader::{Compiled, Shader, Main, TargetProvider, Shared, CompilationError};
use crate::gl_call;
use crate::glsl;
use crate::hlist::indexed;
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
pub struct Program<IS, OS, DUS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    object: Object<ProgramAllocator>,
    semantics: ProgramSemantics<IS, OS>,
    defined_uniforms: DUS,
}

impl<IS, OS> std::default::Default for Program<IS, OS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    fn default() -> Self {
        Self { object: Default::default(), semantics: Default::default(), defined_uniforms: () }
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

impl<IS, OS, DUS> Program<IS, OS, DUS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    // consider intoducing no input / output types so this method is not accessible
    pub fn builder<'s, US>(vertex_shader: &'s Main<Vertex, IS, OS, US>) -> Builder<'s, Vertex, IS, OS, DUS, US>
    where
        IS: parameters::Parameters,
        OS: parameters::Parameters,
        DUS: crate::hlist::lhlist::Append,
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

pub struct Data<IS, OS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    inputs: PhantomData<IS>,
    outputs: PhantomData<OS>,
}

impl<IS, OS> Default for Data<IS, OS>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
{
    fn default() -> Self {
        Self { inputs: Default::default(), outputs: Default::default() }
    }
}

pub struct Uniforms<DUS, UUS> {
    defined_uniforms: DUS,
    unmatched_uniforms: PhantomData<UUS>,
}

impl<UUS> Uniforms<(), UUS> {
    fn new() -> Self {
        Self {
            defined_uniforms: (),
            unmatched_uniforms: PhantomData,
        }
    }
}

impl<DUS> Uniforms<DUS, ()> {
    /// Definition a new uniform with specified index
    pub fn define<const INDEX: usize, U>(self, u: U) -> Uniforms<(DUS, U), ()> {
        let Self { defined_uniforms, unmatched_uniforms } = self;
        Uniforms {
            defined_uniforms: (defined_uniforms, u),
            unmatched_uniforms
        }
    }

    pub fn get<const INDEX: usize, U>(&self) -> &U 
    where
        DUS: indexed::lhlist::Get<U>
    {
        let index: &indexed::Indexed<INDEX, U> = self.defined_uniforms.get::<INDEX>();
    }
}

impl<DUS> Uniforms<DUS, ()> {
    /// Add collection of uniforms 
    pub fn add_unmatched<UUS>(self) -> Uniforms<DUS, UUS> {
        Uniforms {
            defined_uniforms: self.defined_uniforms,
            unmatched_uniforms: PhantomData,
        }
    }
}

pub struct Builder<'shaders, T, IS, OS, DUS, UUS>
where
    T: shader::Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append,
{
    _target_phantom: PhantomData<T>,
    _data: Data<IS, OS>,
    uniforms: Uniforms<DUS, UUS>,

    vertex: ShaderStage<'shaders, Vertex>,
    tesselation_control: Option<ShaderStage<'shaders, tesselation::Control>>,
    tesselation_evaluation: Option<ShaderStage<'shaders, tesselation::Evaluation>>,
    geometry: Option<ShaderStage<'shaders, Geometry>>,
    fragment: Option<ShaderStage<'shaders, Fragment>>,
    compute: Option<ShaderStage<'shaders, Compute>>,
}


impl<'s, T, IS, OS, DUS, UUS> Builder<'s, T, IS, OS, DUS, UUS>
where
    T: shader::Target,
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append
{
    fn retype<NT: shader::Target, NOS: parameters::Parameters, NUUS>(self) -> Builder<'s, NT, IS, NOS, DUS, NUUS> {
        Builder {
            _target_phantom: PhantomData,
            _data: Default::default(),
            uniforms: self.uniforms,
            vertex: self.vertex,
            tesselation_control: self.tesselation_control,
            tesselation_evaluation: self.tesselation_evaluation,
            geometry: self.geometry,
            fragment: self.fragment,
            compute: self.compute,
        }
    }

    // 3 kinds of API
    // uniform_xyz_default -- assigns default values on creation
    // uniform_xyz_initializer -- assigns values from registered callbacks
    // uniform_xyz -- expects parameter in program creation
    // just build list in builder?

    // TODO: Add where U: Uniform Marker
    pub fn uniform<U>(mut self, value: U) -> Builder<'s, T, IS, OS, DUS, (UUS, U)> {
        self.uniforms = (value, self.uniforms)
    }

    pub fn uniform_1f(location: u32, v0: f32) { }
    pub fn uniform_2f(location: u32, v0: f32, v1: f32) { }
    pub fn uniform_3f(location: u32, v0: f32, v1: f32, v2: f32) { }
    pub fn uniform_4f(location: u32, v0: f32, v1: f32, v2: f32, v3: f32) { }

    pub fn uniform_1i(location: u32, v0: i32) { }
    pub fn uniform_2i(location: u32, v0: i32, v1: i32) { }
    pub fn uniform_3i(location: u32, v0: i32, v1: i32, v2: i32) { }
    pub fn uniform_4i(location: u32, v0: i32, v1: i32, v2: i32, v3: i32) { }
    
    pub fn uniform_1ui(location: u32, v0: u32) { }
    pub fn uniform_2ui(location: u32, v0: u32, v1: u32) { }
    pub fn uniform_3ui(location: u32, v0: u32, v1: u32, v2: u32) { }
    pub fn uniform_4ui(location: u32, v0: u32, v1: u32, v2: u32, v3: u32) { }

    pub fn uniform_1fv(location: u32, value: &[f32]) { }
    pub fn uniform_2fv(location: u32, value: &[f32]) { }
    pub fn uniform_3fv(location: u32, value: &[f32]) { }
    pub fn uniform_4fv(location: u32, value: &[f32]) { }

    pub fn uniform_1iv(location: u32, value: &[i32]) { }
    pub fn uniform_2iv(location: u32, value: &[i32]) { }
    pub fn uniform_3iv(location: u32, value: &[i32]) { }
    pub fn uniform_4iv(location: u32, value: &[i32]) { }

    pub fn uniform_1uiv(location: u32, value: &[u32]) { }
    pub fn uniform_2uiv(location: u32, value: &[u32]) { }
    pub fn uniform_3uiv(location: u32, value: &[u32]) { }
    pub fn uniform_4uiv(location: u32, value: &[u32]) { }

    pub fn uniform_matrix_2fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_3fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_4fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_2x3fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_3x2fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_2x4fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_4x2fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_3x4fv(location: u32, transpose: bool, value: &[f32]) { }
    pub fn uniform_matrix_x3fv(location: u32, transpose: bool, value: &[f32]) { }
}

impl<'s, IS, OS, DUS> Builder<'s, Vertex, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append,
{
    pub fn new<US>(vertex_shader: &'s Main<Vertex, IS, OS, US>) -> Builder<'_, Vertex, IS, OS, DUS, US> {
        Self {
            _target_phantom: PhantomData,
            _data: Default::default(),
            uniforms: Uniforms<DUS, US>::de,
            vertex: ShaderStage::new(vertex_shader),
            tesselation_control: None,
            tesselation_evaluation: None,
            geometry: None,
            fragment: None,
            compute: None,
        }
    }

    /// Attach new vertex shader for linking purposes possibly adding new uniforms.
    pub fn vertex_shared<US>(mut self, shader: &'s Shared<Vertex, US>) -> Builder<'_, Vertex, IS, OS, DUS, US> {
        self.vertex.shared.push(shader);
        self
    }

    pub fn tesselation_control_main<TCO: parameters::Parameters, NUS>(mut self, shader: &'s Main<tesselation::Control, OS, TCO, NUS>) -> Builder<tesselation::Control, IS, TCO, DUS, NUS> {
        self.tesselation_control = Some(ShaderStage::new(shader));
        self.retype()
    }

    pub fn geometry_main<GO: parameters::Parameters, US>(mut self, geometry: &'s Main<Geometry, OS, GO, US>) -> Builder<Geometry, IS, GO, DUS, US> {
        self.geometry = Some(ShaderStage::new(geometry));
        self.retype()
    }

    pub fn fragment_main<FO: parameters::Parameters, US>(mut self, fragment: &'s Main<Fragment, OS, FO, US>) -> Builder<Fragment, IS, FO, DUS, US> {
        self.fragment.replace(ShaderStage::new(fragment));
        self.retype()
    }
}

impl<'s, IS, OS, DUS> Builder<'s, tesselation::Control, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append
{
    pub fn tesselation_control_shared<US>(mut self, shared: &'s Shared<tesselation::Control, US>) -> Builder<'_, tesselation::Control, IS, OS, DUS, US> {
        self.tesselation_control.as_mut().expect("tesselation control was initialized").shared.push(shared);
        self
    }

    pub fn tesselation_evaluation_main<TEO, US>(mut self, shader: &'s Main<tesselation::Evaluation, OS, TEO, US>) -> Builder<tesselation::Evaluation, IS, TEO, DUS, US>
    where
        TEO: parameters::Parameters,
    {    
        self.tesselation_evaluation = Some(ShaderStage::new(shader));
        self.retype()
    }
}

impl<'s, IS, OS, DUS> Builder<'s, tesselation::Evaluation, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append
{
    pub fn tesselation_evaluation_shared<US>(mut self, shared: &'s Shared<tesselation::Evaluation, US>) -> Self {
        self.tesselation_evaluation.as_mut().expect("tesselation evaluation stage was initialized").shared.push(shared);
        self
    }

    pub fn geometry_main<GO: parameters::Parameters, US>(mut self, geometry: &'s Main<Geometry, OS, GO, US>) -> Builder<Geometry, IS, GO, DUS, US> {
        self.geometry = Some(ShaderStage::new(geometry));
        self.retype()
    }

    pub fn fragment_main<FO: parameters::Parameters, US>(mut self, fragment: &'s Main<Fragment, OS, FO, US>) -> Builder<Fragment, IS, FO, DUS, US> {
        self.fragment = Some(ShaderStage::new(fragment));
        self.retype()
    }
}

impl<'s, IS, OS, DUS> Builder<'s, Geometry, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append
{
    pub fn geometry_shared<US>(mut self, shared: &'s Shared<Geometry, US>) -> Builder<'_, Geometry, IS, OS, DUS, US> {
        self.geometry.as_mut().expect("geometry stage was initialized").shared.push(shared);
        self
    }

    pub fn fragment_main<FO: parameters::Parameters, US>(mut self, fragment: &'s Main<Fragment, OS, FO, US>) -> Builder<Fragment, IS, FO, DUS, US> {
        self.fragment = Some(ShaderStage::new(fragment));
        self.retype()
    }
}

impl<'s, IS, OS, DUS> Builder<'s, Fragment, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append
{
    pub fn fragment_shared<US>(mut self, shared: &'s Shared<Fragment, US>) -> Builder<'_, Fragment, IS, OS, DUS, US> {
        self.fragment.as_mut().expect("fragment stage was initialized").shared.push(shared);
        self
    }

    pub fn build(&self) -> Result<Program<IS, OS, DUS>, LinkingError> {
        let program = Program::default();
        program.attach(&self.vertex);

        if let (Some(control_stage), Some(evaluation_stage)) = (&self.tesselation_control, &self.tesselation_evaluation) {
            program.attach(control_stage);
            program.attach(evaluation_stage);
        }

        if let Some(geometry) = &self.geometry {
            program.attach(geometry);
        }

        if let Some(fragment) = &self.fragment {
            program.attach(fragment);
        }

        program.link()
    }
}

impl<'s, IS, OS, DUS> Builder<'s, Vertex, IS, OS, DUS, ()>
where
    IS: parameters::Parameters,
    OS: parameters::Parameters,
    DUS: crate::hlist::lhlist::Append
{
    
}
