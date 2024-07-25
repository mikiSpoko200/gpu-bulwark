use crate::{gl::uniform, ts};

use super::shader;

/// Collection of shaders for given program stage with defined stage interface.
///
/// It contains exactly one shaders that contains main function
/// and arbitrary many that are there just to supply shaders to link against.
pub(super) struct ShaderStage<'shaders, T>
where
    T: shader::target::Target,
{
    pub main: &'shaders shader::ShaderObject<T>,
    pub libs: Vec<&'shaders shader::ShaderObject<T>>,
}

impl<'s, T> ShaderStage<'s, T>
where
    T: shader::target::Target,
{
    pub fn new<Decls>(main: &'s shader::Shader<ts::Compiled, T, Decls>) -> ShaderStage<'s, T>
    where 
        Decls: uniform::bounds::Declarations,
    {
        ShaderStage {
            main: main.as_ref(),
            libs: Vec::new(),
        }
    }
}
