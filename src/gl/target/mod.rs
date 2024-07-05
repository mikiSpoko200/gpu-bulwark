/// Definitions of opengl bind targets

/// Common behavior amongst all object specific targets
pub trait Target {
    const VALUE: u32;
}
