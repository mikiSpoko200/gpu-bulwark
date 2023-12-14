# Discussion on Program design

## Program

### Requirements

- Program consists of multiple [`ShaderStage`](##Shader-Stages)s - one for each pipeline stage
- Ownership for program object is described in specification. Most notable aspects are uniforms (shader storage buffers?, atomic counter buffers?)

## Shader Stages

These types represent Shaders for different stages of OpenGL pipeline.

### Requirements

- `ShaderStage<Inputs, Outputs, Stage>` contains at least one `CompiledShader<Stage>`
- `Inputs` and `Outputs` are respectively inputs to `Vertex` stage and output of `Fragment` stage. Note that current definition `ShaderStage` is too general as other stages beside `Vertex` and `Fragment` also need to specify thier inputs / ouputs. To solve this introduce separate types for typed / untyped ins and outs.

### Issues

**`Attach<T>` bound issues**

The purpose of trait `Attach<T>` is to allow users to call `program.attach(&shader)` with arbitrary shader type in fashion similar to C++'s function overloading.
But the bound `T: shader::Target` doesn't really make sense since for tesselation shaders we'd like to expect them to be attached at the same time as for vertex and fragment shaders. This necesitates the creation of new trait that will server as bound on collections of shaders.

Then we can enforce the passing of vertex and fragment shaders in `create` for program and then allow for swapping of these shaders at will since `create` will have different signature then `attach`.

I needed to add `Main<T, I, O>` type that wrapps `CompiledShader<T>` its the point at which user needs to specify shader inputs and outputs. This messes up the attach API. At the same time it begs for TypeState with regard to I/O