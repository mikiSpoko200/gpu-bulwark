/// Definitions of opengl bind targets

mod private {
    /// Common behavior amongst all object specific targets
    pub trait Target {
        const ID: u32;
    }
}

pub(crate) use private::Target;


#[macro_export]
#[allow(unused)]
macro_rules! impl_target {
    ($target_type:ty as $gl_target_ident:ident) => {
        impl $crate::gl::target::Target for $target_type {
            const ID: u32 = glb::$gl_target_ident;
        }
    };
}
