use super::prelude::*;
use crate::object::resource::{Handle, Resource};
use crate::prelude::*;
use crate::{gl_call, impl_const_super_trait};
use gl::types::GLenum;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;

use crate::object::resource;
use thiserror;

pub trait Stage: Const<GLenum> {}

/// Zero-sized struct that represents Vertex Shader stage.
pub struct Vertex;

pub mod tesselation {
    /// Zero-sized struct that represents Tesselation Control Shader stage.
    pub struct Control;

    /// Zero-sized struct that represents Tesselation Evaluation Shader stage.
    pub struct Evaluation;
}

/// Zero-sized struct that represents Geometry Shader stage.
pub struct Geometry;

/// Zero-sized struct that represents Fragment Shader stage.
pub struct Fragment;

/// Zero-sized struct that represents Compute Shader stage.
pub struct Compute;

impl_const_super_trait!(Stage for Vertex, gl::VERTEX_SHADER);
impl_const_super_trait!(Stage for tesselation::Control, gl::TESS_CONTROL_SHADER);
impl_const_super_trait!(Stage for tesselation::Evaluation, gl::TESS_EVALUATION_SHADER);
impl_const_super_trait!(Stage for Geometry, gl::GEOMETRY_SHADER);
impl_const_super_trait!(Stage for Fragment, gl::FRAGMENT_SHADER);
impl_const_super_trait!(Stage for Compute, gl::COMPUTE_SHADER);

pub trait CompilationStatus {}

pub struct Uncompiled;
impl CompilationStatus for Uncompiled {}

pub struct Compiled;
impl CompilationStatus for Compiled {}

pub struct Shader<S, C = Uncompiled>
where
    S: Stage,
    C: CompilationStatus,
{
    base: Object<Self>,
    _stage_phantom: PhantomData<S>,
    _uncompiled_phantom: PhantomData<C>,
}

#[repr(u32)]
pub enum QueryParam {
    ShaderType = gl::SHADER_TYPE,
    DeleteStatus = gl::DELETE_STATUS,
    CompileStatus = gl::COMPILE_STATUS,
    InfoLogLength = gl::INFO_LOG_LENGTH,
    ShaderSourceLength = gl::SHADER_SOURCE_LENGTH,
}

#[derive(thiserror::Error, Debug)]
#[error("shader compilation failed {msg}")]
pub struct CompilationError {
    msg: String,
}

impl CompilationError {
    pub fn new(msg: String) -> Self {
        Self { msg }
    }
}

impl<S> Shader<S, Uncompiled>
where
    S: Stage,
{
    pub fn create() -> Self {
        Self {
            base: Object::default(),
            _stage_phantom: PhantomData,
            _uncompiled_phantom: PhantomData,
        }
    }

    /// Add source for shader.
    pub fn source(&self, sources: &[&str]) -> &Self {
        let pointers: Vec<_> = sources.iter()
            .map(|s| s.as_ptr())
            .collect();
        let lengths: Vec<_> = sources.iter()
            .map(|s| s.len())
            .collect();

        gl_call! {
            #[panic]
            unsafe {
                gl::ShaderSource(
                    self.base.name,
                    sources.len() as _,
                    pointers.as_ptr() as _,
                    lengths.as_ptr() as _
                );
            }
        }
        self
    }

    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
            unsafe {
                gl::GetShaderiv(self.base.name, param as _, output);
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
                    gl::GetShaderInfoLog(
                        self.base.name,
                        buffer.capacity() as _,
                        &mut actual_length as *mut _,
                        buffer.as_mut_ptr() as _
                    )
                }
            }
            // GetShaderInfoLog does not account for null terminator in returned length.
            // SAFETY: nothing will panic here so it's safe to set length.
            unsafe {
                buffer.set_len((actual_length + 1) as _);
            }
            // SAFETY: todo will shader compiler should emmit valid ascii?
            unsafe { String::from_utf8_unchecked(buffer) }
        })
    }

    unsafe fn convert(self) -> Shader<S, Compiled> {
        let Self { base, .. } = self;
        let _leak = unsafe { ManuallyDrop::new(base) };
        Shader::<S, Compiled> {
            base: Object::new(_leak.name),
            _stage_phantom: Default::default(),
            _uncompiled_phantom: Default::default(),
        }
    }

    pub fn compile(self) -> Result<Shader<S, Compiled>, CompilationError> {
        gl_call! {
            #[propagate]
            unsafe {
                gl::CompileShader(self.base.name)
            }
        };
        self
            .info_log()
            // SAFETY: we just checked if shader compiled successfully
            .map_or(
                Ok(unsafe { self.convert() }),
                |msg| Err(CompilationError { msg })
            )
    }
}

impl<S, C> Into<Object<Self>> for Shader<S, C>
where
    S: Stage,
    C: CompilationStatus,
{
    fn into(self) -> Object<Self> {
        let Self { base, .. } = self;
        base
    }
}

impl<S> From<Object<Self>> for Shader<S, Uncompiled>
where
    S: Stage,
{
    fn from(base: Object<Self>) -> Self {
        Self {
            base,
            _stage_phantom: Default::default(),
            _uncompiled_phantom: Default::default()
        }
    }
}

impl<S, C> Resource for Shader<S, C>
where
    S: Stage,
    C: CompilationStatus,
{
    type Ok = ();

    fn initialize(names: &mut [Name]) -> crate::error::Result<Self::Ok> {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for name in names {
            *name = unsafe { gl::CreateShader(S::VALUE) };
            // TODO: Check for errors
        }
        Ok(())
    }

    fn free(names: &[Name]) -> crate::error::Result<Self::Ok> {
        // UNSAFE: Check for 0 return type, otherwise Stage guarantees valid Enum value.
        for name in names {
            unsafe { gl::DeleteShader(*name) };
            // TODO: Check for errors
        }
        Ok(())
    }
}

impl<S> Shader<S, Compiled>
where
    S: Stage,
{
    pub fn inputs() {}
    pub fn outputs() {}
}

pub mod program {
    use crate::object::prelude::Object;
    use crate::object::resource::Resource;
    use crate::object::shader::{tesselation, Compute, Fragment, Geometry, Shader, Stage, Vertex, Uncompiled, Compiled};

    type CompiledShader<S> = Shader<S, Compiled>;

    // Sealed trait
    trait Attach<S>
    where
        S: Stage,
    {
        fn attach(&mut self, shader: CompiledShader<S>);
    }

    pub struct ProgramBuilder {
        base: Object<Program>,
        vertex: Option<CompiledShader<Vertex>>,
        tesselation_control: Option<CompiledShader<tesselation::Control>>,
        tesselation_evaluation: Option<CompiledShader<tesselation::Evaluation>>,
        geometry: Option<CompiledShader<Geometry>>,
        fragment: Option<CompiledShader<Fragment>>,
        compute: Option<CompiledShader<Compute>>,
    }

    impl Attach<Vertex> for ProgramBuilder {
        fn attach(&mut self, vertex: CompiledShader<Vertex>) {
            self.vertex = Some(vertex);
        }
    }

    impl Attach<tesselation::Control> for ProgramBuilder {
        fn attach(&mut self, tesselation_control: CompiledShader<tesselation::Control>) {
            self.tesselation_control = Some(tesselation_control);
        }
    }

    impl Attach<tesselation::Evaluation> for ProgramBuilder {
        fn attach(&mut self, tesselation_evaluation: CompiledShader<tesselation::Evaluation>) {
            self.tesselation_evaluation = Some(tesselation_evaluation);
        }
    }

    impl Attach<Geometry> for ProgramBuilder {
        fn attach(&mut self, geometry: CompiledShader<Geometry>) {
            self.geometry = Some(geometry);
        }
    }

    impl Attach<Fragment> for ProgramBuilder {
        fn attach(&mut self, fragment: CompiledShader<Fragment>) {
            self.fragment = Some(fragment);
        }
    }

    impl Attach<Compute> for ProgramBuilder {
        fn attach(&mut self, compute: CompiledShader<Compute>) {
            self.compute = Some(compute);
        }
    }

    pub struct ProgramConfiguration {}

    impl Default for ProgramConfiguration {
        fn default() -> Self {
            todo!()
        }
    }

    /// Representation of OpenGL Program Object
    ///
    /// Program object by default is in some state -- default?
    /// Program encompasses multiple shader stages.
    /// It can have multiple shaders for the same stage attached to itself
    /// as well as one shader can be attached to multiple programs
    ///
    /// Each Stage has an interface. In order for program to be correct there must more or less match.
    /// One exception that comes to mind is using constant attribute input.
    /// There are rules that govern if two interfaces match
    /// Initially I will consider only matching by using the location specifier since it can
    /// be encoded in type easily with tuples.
    /// Match by parameter name will be difficult to encode in type system, compile time check maybe?
    /// Similarly parameter qualification may be painful and realllly verbose but perhaps default
    /// type parameters will do the trick -- I need to delve into GLSL spec a bit more.
    ///
    /// Programs have associated lists of resources that they use.
    /// These lists seem to be good starting point for modelling the type.
    /// There are multiple program interfaces, here are some more notable ones:
    /// - UNIFORM corresponds to the set of active uniform variables used by program.
    /// - UNIFORM_BLOCK corresponds to the set of active uniform blocks used by program.
    /// - ATOMIC_COUNTER_BUFFER corresponds to the set of active atomic counter buffer binding points used by program.
    /// - PROGRAM_INPUT corresponds to the set of active input variables used by the
    /// first shader stage of program. If program includes multiple shader stages,
    /// input variables from any shader stage other than the first will not be enumerated.
    /// - PROGRAM_OUTPUT corresponds to the set of active output variables used by the
    /// last shader stage of program. If program includes multiple shader stages,
    /// output variables from any shader stage other than the last will not be enumerated.
    /// - BUFFER_VARIABLE corresponds to the set of active buffer variables used by program
    /// - SHADER_STORAGE_BLOCK corresponds to the set of active shader storage blocks used by program
    ///
    /// This represents an ownership model of sorts though things might be different
    /// when using separable programs.
    pub struct Program {
        base: Object<Self>,
        config: ProgramConfiguration,
        vertex: CompiledShader<Vertex>,
        tesselation: Option<(
            CompiledShader<tesselation::Control>,
            CompiledShader<tesselation::Evaluation>,
        )>,
        geometry: Option<CompiledShader<Geometry>>,
        fragment: CompiledShader<Fragment>,
        compute: Option<CompiledShader<Compute>>,
    }

    impl Resource for Program {
        type Ok = ();

        fn initialize(names: &mut [gl::types::GLuint]) -> crate::error::Result<Self::Ok> {
            todo!()
        }

        fn free(names: &[gl::types::GLuint]) -> crate::error::Result<Self::Ok> {
            todo!()
        }
    }
}

pub struct Compiler;

pub struct Linker;

pub fn make<S>() -> Handle<Shader<S, Uncompiled>>
where
    S: Stage,
{
    Handle::default()
}
