use gl;
use gl::types::GLenum;
use std::fmt::Debug;
use thiserror;

#[allow(unused)]
pub type Result<Ok> = std::result::Result<Ok, Box<[Error]>>;

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("invalid enum")]
    InvalidEnum,
    #[error("invalid value")]
    InvalidValue,
    #[error("invalid operation")]
    InvalidOperation,
    #[error("stack overflow")]
    StackOverflow,
    #[error("stack underflow")]
    StackUnderflow,
    #[error("out of memory")]
    OutOfMemory,
    #[error("invalid framebuffer operation")]
    InvalidFramebufferOperation,
    #[error("context lost")]
    ContextLost,
}

impl Error {
    pub fn new(error_code: GLenum) -> Self {
        match error_code {
            gl::INVALID_ENUM => Self::InvalidEnum,
            gl::INVALID_VALUE => Self::InvalidValue,
            gl::INVALID_OPERATION => Self::InvalidOperation,
            gl::STACK_OVERFLOW => Self::StackOverflow,
            gl::STACK_UNDERFLOW => Self::StackUnderflow,
            gl::OUT_OF_MEMORY => Self::OutOfMemory,
            gl::INVALID_FRAMEBUFFER_OPERATION => Self::InvalidFramebufferOperation,
            gl::CONTEXT_LOST => Self::ContextLost,
            _ => panic!("unsupported OpenGL error code {error_code}"),
        }
    }

    pub fn poll_queue() -> Box<[Self]> {
        let mut errors = vec![];
        loop {
            let error = unsafe { gl::GetError() };
            if error == gl::NO_ERROR {
                break;
            }
            errors.push(Error::new(error));
        }
        errors.into_boxed_slice()
    }
}
