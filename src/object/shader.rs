use std::marker::PhantomData;
use gl::types::GLenum;
use crate::object::resource::Resource;
use super::prelude::*;
use crate::prelude::*;

pub trait Stage: Const<GLenum> { }

pub struct Vertex;
pub mod tesselation {
    pub struct Control;
    pub struct Evaluation;
}
pub struct Geometry;
pub struct Fragment;
pub struct Compute;


impl Const<GLenum> for Vertex { const VALUE: GLenum = gl::VERTEX_SHADER; }
impl Stage for Vertex { }

impl Const<GLenum> for tesselation::Control { const VALUE: GLenum = gl::TESS_CONTROL_SHADER; }
impl Stage for tesselation::Control { }

impl Const<GLenum> for tesselation::Evaluation { const VALUE: GLenum = gl::TESS_EVALUATION_SHADER; }
impl Stage for tesselation::Evaluation { }

impl Const<GLenum> for Geometry { const VALUE: GLenum = gl::GEOMETRY_SHADER; }
impl Stage for Geometry { }

impl Const<GLenum> for Fragment { const VALUE: GLenum = gl::FRAGMENT_SHADER; }
impl Stage for Fragment { }

impl Const<GLenum> for Compute { const VALUE: GLenum = gl::COMPUTE_SHADER; }
impl Stage for Compute { }


pub struct Shader<S> where S: Stage {
    base: Object,
    _stage_phantom: PhantomData<S>
}

impl<S> Into<Object> for Shader<S> where S: Stage {
    fn into(self) -> Object {
        let Self { base, .. } = self;
        base
    }
}

impl<S> From<Object> for Shader<S> where S: Stage {
    fn from(base: Object) -> Self {
        Self { base, _stage_phantom: Default::default() }
    }
}

impl<S> Resource for Shader<S> where S: Stage {
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


impl<S> Shader<S> where S: Stage {
    pub fn inputs() { }
    pub fn outputs() { }
}


pub mod program {
    use crate::object::prelude::Object;
    use crate::object::shader::{Compute, Fragment, Geometry, Shader, Stage, tesselation, Vertex};

    // Sealed trait
    trait Attach<S> where S: Stage {
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

    pub struct ProgramConfiguration { }

    impl Default for ProgramConfiguration {
        fn default() -> Self {
            todo!()
        }
    }

    pub struct Program {
        base: Object,
        config: ProgramConfiguration,
        vertex: Shader<Vertex>,
        tesselation: Option<(Shader<tesselation::Control>, Shader<tesselation::Evaluation>)>,
        geometry: Option<Shader<Geometry>>,
        fragment: Shader<Fragment>,
        compute: Option<Shader<Compute>>,
    }
}


pub struct Compiler;

pub struct Linker;
