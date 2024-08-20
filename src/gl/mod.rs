#![allow(unused)]

use crate::prelude::internal::*;

pub mod buffer;
pub mod program;
pub mod object;
pub mod shader;
pub mod texture;
pub mod vertex_array;
pub mod uniform;
pub mod types;
pub mod valid;
pub mod bounds;
pub mod image;
pub mod error;

// Reexports
pub use types::*;

pub(crate) mod target;

pub use buffer::Buffer;
pub use program::Program;
pub use vertex_array::{VertexArray, VAO};

use crate::glsl;
use crate::gl;
use glsl::storage::{In, Out};
use object::Binder;
 

pub type Result<T> = std::result::Result<T, Box<[error::Error]>>;


pub fn draw_arrays<AS, PSI, PSO, US>(vao: &vertex_array::VertexArray<AS>, program: &Program<PSI, PSO, US>)
where
    AS: vertex_array::valid::Attributes,
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
            let errors = $crate::gl::error::Error::poll_queue();
            if errors.len() > 0 {
                let message = errors.iter().map(ToString::to_string).collect::<::std::vec::Vec<_>>().join("\n");
                panic!("gl error: {message}");
            }
        }
    };
    ([propagate] $invocation:stmt) => {
        $invocation
        let errors = $crate::gl::error::Error::poll_queue();
        if errors.len() > 0 { Err(errors) } else { Ok(()) }
    };
}

pub(crate) use call;

macro_rules! impl_token {
    ($ty:ty as $token_trait:path => $gl_token_name:ident) => {
        impl $token_trait for $ty {
            const ID: u32 = ::glb::$gl_token_name;
        }
    };
}

pub(crate) use impl_token;