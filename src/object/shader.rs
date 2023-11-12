use super::prelude::*;
use crate::object::resource::Resource;
use crate::prelude::*;
use crate::{gl_call, impl_const_super_trait};
use gl::types::GLenum;
use std::marker::PhantomData;

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

pub struct Shader<S>
where
    S: Stage,
{
    base: Object,
    _stage_phantom: PhantomData<S>,
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

impl<S> Shader<S>
where
    S: Stage,
{
    pub unsafe fn source_from_raw(&mut self, source: &[*const u8]) {
        gl_call! {
            #[panic]
            unsafe {
                gl::ShaderSource(self.base.0, 1, source.as_ptr() as _, std::ptr::null());
            }
        }
    }

    /// Create new shader from source
    pub fn source(source: &[&str]) -> Result<Self, CompilationError> {
        let mut result: Self = super::resource::manager::create();
        let raw_strings: Vec<_> = source.into_iter().map(|input| input.as_ptr()).collect();
        unsafe { result.source_from_raw(&raw_strings) };
        result
            .info_log()
            .map_or(Ok(result), |msg| Err(CompilationError { msg }))
    }

    pub fn query(&self, param: QueryParam, output: &mut i32) {
        gl_call! {
            #[panic]
            unsafe {
                gl::GetShaderiv(self.base.0, param as _, output);
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
                        self.base.0,
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
}

impl<S> Into<Object> for Shader<S>
where
    S: Stage,
{
    fn into(self) -> Object {
        let Self { base, .. } = self;
        base
    }
}

impl<S> From<Object> for Shader<S>
where
    S: Stage,
{
    fn from(base: Object) -> Self {
        Self {
            base,
            _stage_phantom: Default::default(),
        }
    }
}

impl<S> Resource for Shader<S>
where
    S: Stage,
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

impl<S> Shader<S>
where
    S: Stage,
{
    pub fn inputs() {}
    pub fn outputs() {}
}

pub mod program {
    use crate::object::prelude::Object;
    use crate::object::shader::{tesselation, Compute, Fragment, Geometry, Shader, Stage, Vertex};

    // Sealed trait
    trait Attach<S>
    where
        S: Stage,
    {
        fn attach(&mut self, shader: Shader<S>);
    }

    pub struct ProgramBuilder {
        base: Object,
        vertex: Option<Shader<Vertex>>,
        tesselation_control: Option<Shader<tesselation::Control>>,
        tesselation_evaluation: Option<Shader<tesselation::Evaluation>>,
        geometry: Option<Shader<Geometry>>,
        fragment: Option<Shader<Fragment>>,
        compute: Option<Shader<Compute>>,
    }

    impl Attach<Vertex> for ProgramBuilder {
        fn attach(&mut self, vertex: Shader<Vertex>) {
            self.vertex = Some(vertex);
        }
    }

    impl Attach<tesselation::Control> for ProgramBuilder {
        fn attach(&mut self, tesselation_control: Shader<tesselation::Control>) {
            self.tesselation_control = Some(tesselation_control);
        }
    }

    impl Attach<tesselation::Evaluation> for ProgramBuilder {
        fn attach(&mut self, tesselation_evaluation: Shader<tesselation::Evaluation>) {
            self.tesselation_evaluation = Some(tesselation_evaluation);
        }
    }

    impl Attach<Geometry> for ProgramBuilder {
        fn attach(&mut self, geometry: Shader<Geometry>) {
            self.geometry = Some(geometry);
        }
    }

    impl Attach<Fragment> for ProgramBuilder {
        fn attach(&mut self, fragment: Shader<Fragment>) {
            self.fragment = Some(fragment);
        }
    }

    impl Attach<Compute> for ProgramBuilder {
        fn attach(&mut self, compute: Shader<Compute>) {
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
        base: Object,
        config: ProgramConfiguration,
        vertex: Shader<Vertex>,
        tesselation: Option<(
            Shader<tesselation::Control>,
            Shader<tesselation::Evaluation>,
        )>,
        geometry: Option<Shader<Geometry>>,
        fragment: Shader<Fragment>,
        compute: Option<Shader<Compute>>,
    }
}

pub struct Compiler;

pub struct Linker;
