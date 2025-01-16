#![allow(unused)]

use crate::Ctx;

use gb::gl::vertex_array::Attribute;
use gb::glsl::MatchingInputs;
use winit::window;
use glutin::{context, surface};
use gb::gl::{self, VertexArray};
use gl::buffer::target;
use gl::texture;
use gl::shader;
use gb::glsl;
use std::io::Write;
use gl::buffer::{Dynamic, Draw};
use gl::{Program, Buffer};

type VertexShaderInputs = glsl::Glsl! {
    layout(location = 0) in vec3;
    layout(location = 1) in vec3;
};

type VertexShaderOutputs = glsl::Glsl! {
    layout(location = 0) out vec4 color;
};

type FragmentShaderOutputs = glsl::Glsl! {
    layout(location = 0) out vec4;
};

pub type Color = [f32; 3];
pub type Position = [f32; 3];

type Attributes = gb::HList! {
    Attribute<Color   , 0>,
    Attribute<Position, 1>,
};

pub struct Sample {
    program: Program<VertexShaderInputs, FragmentShaderOutputs, (), ()>,
    vao: VertexArray<Attributes>,
}

impl Sample {
    // Color values will be shifted by this much with each key press
    const ATTENUATION_FACTOR: f32 = 0.005;
}

fn initialize() -> anyhow::Result<()> {
    
    // Read shader source code.
    let vs_source = std::fs::read_to_string("shaders/hello_vertices.vert")?;
    let fs_source = std::fs::read_to_string("shaders/hello_vertices.frag")?;

    // GLSL varaible bindings.
    let vs_inputs  = VertexShaderInputs::default();
    let vs_outputs = VertexShaderOutputs::default();
    let fs_inputs  = vs_outputs.matching_inputs();
    let fs_outputs = FragmentShaderOutputs::default();

    // Unpacking type level lists of variables.
    let glsl::vars![ fs_output ] = fs_outputs;
    let glsl::vars![ vin_color, vin_position ] = &vs_inputs;

    let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
    let mut uncompiled_fs = shader::create::<shader::target::Fragment>();

    uncompiled_vs.source(&[&vs_source]);
    uncompiled_fs.source(&[&fs_source]);

    // Defining shaders.
    let vs = uncompiled_vs
        .compile()?
        .into_main()
        .inputs(&vs_inputs)
        .outputs(&vs_outputs);
    let fs = uncompiled_fs
        .compile()?
        .into_main()
        .inputs(&fs_inputs)
        .output(&fs_output);

    // Building type checked pipeline.
    let program = Program::builder()
        .no_uniforms()
        .no_resources()
        .vertex_main(&vs)
        .fragment_main(&fs)
        .build()?;

    let mut colors = Buffer::create();
    colors.data::<(Dynamic, Draw)>(&[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0f32]]);

    let mut positions = Buffer::create();
    positions.data::<(Dynamic, Draw)>(&[[-0.5, -0.5, -1.0], [0.5, -0.5, -1.0], [0.0, 0.5, -1.0f32]]);


    // Vertex attribute array configuration.
    let vao = VertexArray::create()
        .vertex_attrib_pointer(&vin_color, colors)
        .vertex_attrib_pointer(&vin_position, positions)
        ;
}

pub fn try_using_uncompiled() {
    let mut shader = shader::create::<shader::target::Vertex>();
    shader.into_main();
}

pub fn overlapping_locations() {
    let glsl::vars! [ mvp_matrix, offset ] = gb::glsl! {
        layout(location = 0) in mat4;
        layout(location = 4) in vec4;
    };
}
