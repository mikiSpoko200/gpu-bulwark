#![allow(unused)]

use crate::prelude::internal::*;

pub mod buffer;
pub mod program;
pub mod object;
pub mod shader;
pub mod texture;
pub mod vertex_array;
pub mod uniform;
pub mod primitive;
pub mod valid;
pub mod bounds;

// Reexports
pub use primitive::*;

pub(crate) mod target;

use crate::glsl;
use crate::gl;
use glsl::storage::{In, Out};
use program::Program;
use object::Binder;
use vertex_array::VertexArray;
 

pub fn draw_arrays<AS, PSI, PSO, US>(vao: &vertex_array::VertexArray<AS>, program: &Program<PSI, PSO, US>)
where
    AS: glsl::valid::Attributes,
    PSI: glsl::Parameters<In>,
    PSO: glsl::Parameters<Out>,
    AS: glsl::compatible::hlist::Compatible<PSI>,
    US: uniform::bounds::Declarations,
{
    let _vao_bind = vao.bind();
    let _program_bind = program.bind();

    gl::call! {
        [panic]
        unsafe {
            glb::DrawArrays(glb::TRIANGLES, 0, vao.len() as _);
        }
    }
}

/// Wrapper for calling opengl functions.
///
/// In Debug mode it checks for errors and panics.
/// In Release it does nothing.
#[allow(unused)]
macro_rules! call {
    ([panic] $invocation:stmt) => {
        $invocation
        if cfg!(debug_assertions) {
            let errors = $crate::error::Error::poll_queue();
            if errors.len() > 0 {
                let message = errors.into_iter().map(ToString::to_string).collect::<::std::vec::Vec<_>>().join("\n");
                panic!("gl error: {message}");
            }
        }
    };
    ([propagate] $invocation:stmt) => {
        $invocation
        let errors = $crate::error::Error::poll_queue();
        if errors.len() > 0 { Err(errors) } else { Ok(()) }
    };
}

pub(crate) use call;
