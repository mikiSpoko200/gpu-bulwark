#[macro_export]
#[allow(unused)]
macro_rules! impl_target {
    ($target_mod:ident, $target_type:ty, $enum_const:ident) => {
        unsafe impl crate::targets::$target_mod::Target for $target_type {
            const BIND_TARGET: crate::gl::types::GLenum = crate::gl::$enum_const;
        }
    };
}
