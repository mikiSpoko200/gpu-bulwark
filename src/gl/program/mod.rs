pub mod builder;
pub mod stage;

use crate::glsl::bounds::OpaqueUniform;
use crate::glsl::storage::Out;
use crate::glsl::variable::SamplerVariable;
use crate::prelude::internal::*;

use crate::ts;

pub use builder::Builder;

use crate::gl;
use crate::glsl;
use crate::hlist;
use crate::hlist::counters::Index;
use crate::hlist::indexed;
use hlist::lhlist::Find;
use crate::valid;

use gl::object::*;
use gl::shader;
use gl::shader::prelude::*;
use gl::uniform;
use gl::vertex_array;
use glsl::variable;

use variable::TransparentUniformVariable;
use variable::{layout, storage};

use super::texture;
use super::texture::TextureState;
use super::texture::TextureUnit;
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

struct ProgramState<Ins, Outs, Unis, Smpls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Unis: uniform::bounds::Declarations,
{
    pub _phantoms: PhantomData<(Ins, Outs, Smpls)>,
    pub uniform_declarations: uniform::Declarations<ts::Immutable, Unis>,
}

impl<Ins, Outs, Smpls> Default for ProgramState<Ins, Outs, (), Smpls>
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

impl<Ins, Outs, Unis, Smpls> ProgramState<Ins, Outs, Unis, Smpls>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Unis: uniform::bounds::Declarations,
{
    pub fn new(decls: uniform::Declarations<ts::Mutable, Unis>) -> Self {
        Self {
            _phantoms: PhantomData,
            uniform_declarations: decls.into_immutable(),
        }
    }
}

#[derive(dm::Deref)]
#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<Ins, Outs, Unis, Res /* required external resources */>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Unis: uniform::bounds::Declarations,
{
    #[deref]
    object: ObjectBase<ProgramObject>,
    state: ProgramState<Ins, Outs, Unis, Res>,
}

impl Program<(), (), (), ()> {
    pub fn builder<'s>() -> Builder<'s, ts::None, (), (), (), (), ()> {
        Builder::new()
    }
}

impl<Ins, Outs> Program<Ins, Outs, (), ()>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
{
    pub fn create_with_uniforms<Defs, Smpls>(definitions: &uniform::Definitions<Defs>) -> Program<Ins, Outs, Defs::AsDeclarations, Smpls>
    where
        Defs: uniform::bounds::Definitions,
    {
        Program {
            object: Default::default(),
            state: ProgramState::new(Declarations(PhantomData)),
        }
    }
}

impl<Ins, Outs, Unis, Res> Program<Ins, Outs, Unis, Res>
where
    Ins: glsl::Parameters<storage::In>,
    Outs: glsl::Parameters<storage::Out>,
    Unis: uniform::bounds::Declarations,
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

    pub(self) fn link(self) -> Result<Program<Ins, Outs, Unis, Res>, LinkingError> {
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
        var: &TransparentUniformVariable<GLSL, LOCATION>,
        uniform: &impl glsl::Compatible<GLSL>,
    ) where
        GLSL: glsl::bounds::TransparentUniform,
        IDX: Index,
        Unis: Find<TransparentUniformVariable<GLSL, LOCATION>, IDX>,
    {
        self.bound(|_binder| GLSL::set(_binder, var, uniform));
    }

    pub fn draw_arrays_ext<Attrs, Handles>(&self, vao: &gl::VertexArray<Attrs>, _: &texture::TextureUnits<Handles>)
    where
        Attrs: vertex_array::valid::Attributes + glsl::compatible::hlist::Compatible<Ins>,
        Handles: ResourceProviders<Res>,
    {
        let _vao_bind = vao.bind();
        let _program_bind = self.bind();

        gl::call! {
            [panic]
            unsafe {
                glb::DrawArrays(glb::TRIANGLES, 0, vao.len() as _);
            }
        }
    }
}

/// Resource external to the Program, which program can use like textures, images, atomic counters, buffers etc.
pub trait Resource {
    type UniformVariable: OpaqueUniform;

    fn opaque_uniform_variable<const BINDING: usize>(&self) -> glsl::variable::OpaqueUniformVariable<Self::UniformVariable, BINDING>;
}

pub(crate) mod private {
    use super::*;
    pub trait Sealed { }
    
    impl Sealed for () { }
    impl<'texture, TUH, Target, Kind, InternalFormat, const BINDING: usize> private::Sealed for (TUH, &'texture TextureUnit<Target, Kind, InternalFormat, BINDING>)
    where 
        Target: texture::Target,
        Kind: texture::storage::marker::Kind<Target = Target>,
        InternalFormat: texture::image::marker::Format,
    { }
}

#[hi::marker]
pub trait ResourceProviders<Res>: private::Sealed { }

impl ResourceProviders<()> for () {}
impl<'texture, Target, Kind, InternalFormat, const BINDING: usize> ResourceProviders<((), glsl::variable::SamplerVariable<Target, InternalFormat::Output, BINDING>)> for ((), &'texture TextureUnit<Target, Kind, InternalFormat, BINDING>)
where
    Target: texture::Target,
    Kind: texture::storage::marker::Kind<Target = Target>,
    InternalFormat: texture::image::marker::Format,
{ }

impl<'texture, RH, TUH, PrevTarget, PrevKind, PrevInternalFormat, const PREV_BINDING: usize, CurrTarget, CurrKind, CurrInternalFormat, const CURR_BINDING: usize> 
ResourceProviders<((RH, glsl::variable::SamplerVariable<PrevTarget, PrevInternalFormat::Output, PREV_BINDING>), glsl::variable::SamplerVariable<CurrTarget, CurrInternalFormat::Output, CURR_BINDING>)> for
                  ((TUH, &'texture TextureUnit<PrevTarget, PrevKind, PrevInternalFormat, PREV_BINDING>       ), &'texture TextureUnit<CurrTarget, CurrKind, CurrInternalFormat, CURR_BINDING>)
where
    RH: glsl::bounds::OpaqueUniform,
    TUH: ResourceProviders<RH>,
    PrevTarget: texture::Target,
    PrevKind: texture::storage::marker::Kind<Target = PrevTarget>,
    PrevInternalFormat: texture::image::marker::Format,
    CurrTarget: texture::Target,
    CurrKind: texture::storage::marker::Kind<Target = CurrTarget>,
    CurrInternalFormat: texture::image::marker::Format,
{ }

/// Declarations of 'Resource's that program uses.
pub struct Resources<Res>(PhantomData<Res>);

impl Default for Resources<()> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<Res> Resources<Res> {
    /// Add declaration of usage of specified resource.
    pub fn sampler<Target, Output, const BINDING: usize>(
        self, 
        _: &'_ glsl::variable::SamplerVariable<Target, Output, BINDING>
    ) -> Resources<(Res, glsl::variable::SamplerVariable<Target, Output, BINDING>)>
    where
        Target: texture::Target,
        Output: glsl::sampler::Output,
    {
        Resources(PhantomData)
    }
}