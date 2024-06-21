use super::shader;

/// Collection of shaders for given program stage with defined stage interface.
///
/// It contains exactly one shaders that contains main function
/// and arbitrary many that are there just to supply shaders to link against.
pub(super) struct ShaderStage<'shaders, T>
where
    T: shader::target::Target,
{
    pub main: &'shaders shader::internal::CompiledShader<T>,
    pub shared: Vec<&'shaders shader::internal::CompiledShader<T>>,
}

impl<'s, T> ShaderStage<'s, T>
where
    T: shader::target::Target,
{
    pub fn new(main: &'s shader::internal::CompiledShader<T>) -> ShaderStage<'s, T> {
        ShaderStage {
            main,
            shared: Vec::new(),
        }
    }
}
