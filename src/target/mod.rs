/// Definitions of opengl bind targets
pub mod prelude;

/// Common behavior amongst all object specific targets
pub trait Target: Default {
    const VALUE: u32;
}
