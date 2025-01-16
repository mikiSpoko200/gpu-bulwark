/// `glGet` API.

/// Wrapper around OpenGL literals.
pub struct SymbolicConstant<const SYMBOLIC_CONSTANT: glb::types::GLenum>;

pub trait Meta {
    type Get: Get;
}

pub trait Get: Default {
    fn get(&mut self, pname: glb::types::GLenum);
}

impl Get for i32 {
    fn get(&mut self, pname: glb::types::GLenum) {
        gl::call! {
            [panic]
            unsafe {
                glb::
            }
        }
    }
}

macro_rules! symbolic_constant {
    ($($gl_symbolic_constants:ident use $getter:ty),+ $(,)?) => {
        $(
            symbolic_constant!(@constify $gl_symbolic_constants)
            impl Meta for SymbolicConstant<::glb::$gl_symbolic_constants> {
                type Get = $getter;
            }
        )+
    };

    (@constify $gl_symbolic_constants:ident) => {
        impl SymbolicConstant<::glb::$gl_symbolic_constants> {
            const $gl_symbolic_constants = Self(::glb::$gl_symbolic_constants);
        }
    };
}

pub fn get<const SYMBOLIC_CONSTANT: glb::types::GLenum>() -> <SymbolicConstant<SYMBOLIC_CONSTANT> as Meta>::Get
where
    SymbolicConstant<SYMBOLIC_CONSTANT>: Meta
{
    let mut ret = <SymbolicConstant<SYMBOLIC_CONSTANT> as Meta>::Get::default();
    ret.get(SYMBOLIC_CONSTANT);
    ret
}
