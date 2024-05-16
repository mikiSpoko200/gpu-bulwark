#![allow(unused)]

pub mod buffer;
pub mod attributes;
pub mod prelude;
pub mod program;
pub mod resource;
pub mod shader;
pub mod vertex_array;
pub mod texture;

use resource::Bind;
use program::Program;
use vertex_array::VertexArray;
use crate::{gl_call, glsl};
use glsl::prelude::*;

use self::program::uniform;

pub fn draw_arrays<AS, PSI, PSO, US>(vao: &vertex_array::VertexArray<AS>, program: &Program<PSI, PSO, US>)
where
    AS: attributes::Attributes,
    PSI: glsl::Parameters<In>,
    PSO: glsl::Parameters<Out>,
    AS: glsl::compatible::hlist::Compatible<PSI>,
    US: glsl::Uniforms,
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
