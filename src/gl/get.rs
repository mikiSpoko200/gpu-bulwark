/// `glGet` API.

use crate::gl;

/// Wrapper around OpenGL literals.
pub struct SymbolicConstant<const VALUE: glb::types::GLenum>;

mod sealed {
    pub trait GetSymbolicConstant {
        type Value: Get;
        fn get() -> Self::Value;
    }

    pub trait Get: Default + Sized {
        fn get(pname: glb::types::GLenum) -> Self;
    }
}

pub use sealed::*;

macro_rules! impl_get {
    ($ty:ty, $proc:ident) => {
        impl Get for $ty {
            fn get(pname: glb::types::GLenum) -> Self {
                let mut ret = Default::default();
                unsafe {
                    glb::$proc(pname, &raw mut ret);
                }
                ret
            }
        }
    };
}

impl_get! { i32, GetIntegerv }
impl_get! { f32, GetFloatv }
impl_get! { f64, GetDoublev }
impl_get! { i64, GetInteger64v }

// Boolean in special, we convert to rust bool
impl Get for bool {
    fn get(pname: glb::types::GLenum) -> Self {
        let mut gl_bool = Default::default();
        unsafe {
            glb::GetBooleanv(pname, &raw mut gl_bool);
        }
        gl_bool == glb::TRUE
    }
}

macro_rules! symbolic_constant {
    ($getter:ty => [$($gl_symbolic_constants:ident),+ $(,)?]) => {
        $(
            pub const $gl_symbolic_constants: ::glb::types::GLenum = ::glb::$gl_symbolic_constants;
            impl GetSymbolicConstant for SymbolicConstant<{::glb::$gl_symbolic_constants}> {
                type Value = $getter;
                fn get() -> Self::Value {
                    <Self::Value as Get>::get(::glb::$gl_symbolic_constants)
                }
            }
        )+
    };
}

// NOTE: exposing raw consntats is quite bad, would much rather have them as wrapped into [SymbolicConstant] for type safety
//  but then it could create problems with custom const types which is needed for smeamless reimports of identifiers.

symbolic_constant! {
    i32 => [
        ELEMENT_ARRAY_BUFFER_BINDING,
        ARRAY_BUFFER_BINDING
    ]
}

pub fn get<const VALUE: glb::types::GLenum>() -> <SymbolicConstant<VALUE> as GetSymbolicConstant>::Value
where
    SymbolicConstant<VALUE>: GetSymbolicConstant
{
    SymbolicConstant::<VALUE>::get()
}
