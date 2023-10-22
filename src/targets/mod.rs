/// Definitions of opengl bind targets

pub mod buffer;
pub mod prelude;

/// Common behaviour amongst all object specific targets
pub(self) unsafe trait BaseTarget {
    const BIND_TARGET: gl::types::GLenum;
}
