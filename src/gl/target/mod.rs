/// Definitions of opengl bind targets

/// Common behavior amongst all object specific targets
pub trait Target {
    const VALUE: u32;
}

#[macro_export]
#[allow(unused)]
macro_rules! impl_target {
    ($target_type:ty as $gl_target_ident: ident) => {
        impl $crate::gl::target::Target for $target_type {
            const VALUE: u32 = glb::$gl_target_ident;
        }
    };
}
