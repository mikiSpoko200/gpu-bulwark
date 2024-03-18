#![allow(unused)]

pub mod buffer;
pub mod attributes;
pub mod prelude;
pub mod program;
pub mod resource;
pub mod shader;
pub mod vertex_array;

use shader::parameters;
use resource::Bindable;
use program::Program;
use vertex_array::VertexArray;
use crate::{gl_call, glsl};

use self::program::uniform;

pub fn draw_arrays<AS, PSI, PSO, US>(vao: &vertex_array::VertexArray<AS>, program: &Program<PSI, PSO, US>)
where
    AS: attributes::Attributes,
    PSI: parameters::Parameters,
    PSO: parameters::Parameters,
    (AS, PSI): glsl::compatible::Compatible<AS, PSI>,
    US: uniform::marker::Definitions
{
    vao.bind();
    program.bind();

    gl_call! {
        #[panic]
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, vao.len() as _);
        }
    }

    vao.unbind();
    program.unbind();
}
