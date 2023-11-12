pub use gl::types::GLuint as Name;

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Default, Copy, Clone)]
pub struct Object(pub Name);
