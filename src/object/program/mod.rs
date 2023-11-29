use frunk::labelled::chars::T;
use nonempty::{nonempty, NonEmpty};

use super::prelude::Object;
use super::resource::Allocator;
use super::shader::{Compiled, Shader};
use crate::gl_call;
use crate::target::shader;
use crate::target::shader::{tesselation, Compute, Fragment, Geometry, Vertex};
use std::marker::PhantomData;

pub mod layout;

pub type CompiledShader<T> = Shader<T, Compiled>;

pub trait ShaderInterface {}

/// Collection of shaders for given program stage with defined stage interface.
///
/// It contains exactly one shaders that contains main function
/// and arbitrary many that are there just to supply subroutines.
struct ShaderStage<'shaders, T, Inputs, Outputs>
where
    T: shader::Target,
{
    pub shaders: NonEmpty<&'shaders CompiledShader<T>>,
    _in_phantom: PhantomData<Inputs>,
    _out_phantom: PhantomData<Outputs>,
}

impl<'shaders, T, I, O> ShaderStage<'shaders, T, I, O>
where
    T: shader::Target,
{
    pub fn new(shader: &'shaders CompiledShader<T>) -> Self {
        Self {
            shaders: nonempty!(shader),
            _in_phantom: PhantomData,
            _out_phantom: PhantomData,
        }
    }
}

pub trait LinkingStatus {}

pub struct UnLinked;
impl LinkingStatus for UnLinked {}

pub struct Linked;
impl LinkingStatus for Linked {}

pub struct ProgramAllocator;

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

struct ProgramSemantics<'shaders, I, O, L>
where
    L: LinkingStatus,
{
    pub _linking_status: PhantomData<L>,
    pub vertex_stage: ShaderStage<'shaders, Vertex, I, ()>,
    pub tesselation_stage: Option<(
        ShaderStage<'shaders, tesselation::Control, (), ()>,
        ShaderStage<'shaders, tesselation::Evaluation, (), ()>,
    )>,
    pub geometry_stage: Option<ShaderStage<'shaders, Geometry, (), ()>>,
    pub fragment_stage: ShaderStage<'shaders, Fragment, (), O>,
    pub compute_stage: Option<ShaderStage<'shaders, Compute, (), ()>>,
}

impl<'shaders, I, O> ProgramSemantics<'shaders, I, O, UnLinked> {
    pub fn link(self) -> ProgramSemantics<'shaders, I, O, Linked> {
        ProgramSemantics::<I, O, Linked> {
            _linking_status: PhantomData,
            vertex_stage: self.vertex_stage,
            tesselation_stage: self.tesselation_stage,
            geometry_stage: self.geometry_stage,
            fragment_stage: self.fragment_stage,
            compute_stage: self.compute_stage,
        }
    }
}

#[doc = include_str!("../../../docs/object/program/Program.md")]
pub struct Program<'shaders, I, O, L = UnLinked>
where
    L: LinkingStatus,
{
    object: Object<ProgramAllocator>,
    semantics: ProgramSemantics<'shaders, I, O, L>,
}

impl<'shaders, I, O> Program<'shaders, I, O, UnLinked> {
    pub fn link(self) -> Program<'shaders, I, O, Linked> {
        let Self { object, semantics } = self;
        unsafe {
            gl::LinkProgram(object.name());
        }

        Program::<'shaders, _, _, _> {
            object,
            semantics: semantics.link(),
        }
    }
}

/// Marker that specifies what types are sufficient for
pub trait StageConfiguration {}
impl StageConfiguration for CompiledShader<Vertex> {}
impl StageConfiguration for CompiledShader<Fragment> {}
impl StageConfiguration for CompiledShader<Geometry> {}
impl StageConfiguration for CompiledShader<Compute> {}
impl StageConfiguration for CompiledShader<tesselation::Control> {}
impl StageConfiguration for CompiledShader<tesselation::Evaluation> {}
impl StageConfiguration
    for (
        CompiledShader<tesselation::Control>,
        CompiledShader<tesselation::Evaluation>,
    )
{
}

trait Attach<C>
where
    C: StageConfiguration,
{
    type Config;

    fn attach(&mut self, config: Self::Config);
}

impl<'shaders, I, O> Attach<CompiledShader<Vertex>> for Program<'shaders, I, O> {
    type Config = &'shaders CompiledShader<Vertex>;

    fn attach(&mut self, shader: Self::Config) {
        self.semantics.vertex_stage.shaders.push(shader);
    }
}

impl<'shaders, I, O> Attach<CompiledShader<Fragment>> for Program<'shaders, I, O> {
    type Config = &'shaders CompiledShader<Fragment>;

    fn attach(&mut self, shader: Self::Config) {
        self.semantics.fragment_stage.shaders.push(shader);
    }
}

impl<'shaders, I, O> Attach<CompiledShader<Geometry>> for Program<'shaders, I, O> {
    type Config = &'shaders CompiledShader<Geometry>;

    fn attach(&mut self, shader: Self::Config) {
        match self.semantics.geometry_stage {
            Some(ref mut stage) => stage.shaders.push(shader),
            None => self.semantics.geometry_stage = Some(ShaderStage::new(shader)),
        };
    }
}

impl<'shaders, I, O> Attach<CompiledShader<Compute>> for Program<'shaders, I, O> {
    type Config = &'shaders CompiledShader<Compute>;

    fn attach(&mut self, shader: Self::Config) {
        match self.semantics.compute_stage {
            Some(ref mut stage) => stage.shaders.push(shader),
            None => self.semantics.compute_stage = Some(ShaderStage::new(shader)),
        };
    }
}

impl<'shaders, I, O>
    Attach<(
        CompiledShader<tesselation::Control>,
        CompiledShader<tesselation::Evaluation>,
    )> for Program<'shaders, I, O>
{
    type Config = &'shaders (
        CompiledShader<tesselation::Control>,
        CompiledShader<tesselation::Evaluation>,
    );

    fn attach(&mut self, (control, evaluation): Self::Config) {
        match self.semantics.tesselation_stage {
            Some((ref mut old_control, ref mut old_evaluation)) => {
                old_control.shaders.push(control);
                old_evaluation.shaders.push(evaluation);
            }
            None => {
                self.semantics.tesselation_stage =
                    Some((ShaderStage::new(control), ShaderStage::new(evaluation)))
            }
        };
    }
}

impl<'shaders, I, O> Attach<CompiledShader<tesselation::Control>> for Program<'shaders, I, O> {
    type Config = &'shaders CompiledShader<tesselation::Control>;

    fn attach(&mut self, shader: Self::Config) {
        let Some((control, _)) = self.semantics.tesselation_stage.as_mut() else {
            panic!("tesselation stage unattached");
        };
        control.shaders.push(shader);
    }
}

impl<'shaders, I, O> Attach<CompiledShader<tesselation::Evaluation>> for Program<'shaders, I, O> {
    type Config = &'shaders CompiledShader<tesselation::Evaluation>;

    fn attach(&mut self, shader: Self::Config) {
        let Some((_, evaluation)) = self.semantics.tesselation_stage.as_mut() else {
            panic!("tesselation stage unattached");
        };
        evaluation.shaders.push(shader);
    }
}
