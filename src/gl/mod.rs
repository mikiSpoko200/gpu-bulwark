#![allow(unused)]

pub mod buffer;
pub mod program;
pub mod object;
pub mod shader;
pub mod texture;
pub mod vertex_array;
pub mod uniform;

pub(crate) mod target;

use crate::glsl;
use crate::valid;
use crate::gl;
use glsl::prelude::*;
use marker::storage::{In, Out};
use program::Program;
use object::Binder;
use vertex_array::VertexArray;

use crate::prelude::internal::*;

pub fn draw_arrays<AS, PSI, PSO, US>(vao: &vertex_array::VertexArray<AS>, program: &Program<PSI, PSO, US>)
where
    AS: valid::Attributes,
    PSI: glsl::Parameters<In>,
    PSO: glsl::Parameters<Out>,
    AS: glsl::compatible::hlist::Compatible<PSI>,
    US: uniform::bounds::Declarations,
{
    vao.bind();
    program.bind();

    gl::call! {
        [panic]
        unsafe {
            glb::DrawArrays(glb::TRIANGLES, 0, vao.len() as _);
        }
    }

    vao.unbind();
    program.unbind();
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