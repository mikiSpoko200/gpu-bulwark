/// Definitions of opengl bind targets
pub mod prelude;

/// Common behavior amongst all object specific targets
pub unsafe trait Target: Default {
    const VALUE: u32;
}

#[macro_export]
#[allow(unused)]
macro_rules! impl_target {
    ($target_object_module:ident, $target_type:ty, $gl_target_ident:ident) => {
        unsafe impl $crate::target::Target for $target_type {
            const VALUE: u32 = gl::$gl_target_ident;
        }
        unsafe impl $crate::object::$target_object_module::target::Target for $target_type {}
    };
}
