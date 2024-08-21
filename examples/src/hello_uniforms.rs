
use winit::window;
use glutin::{context, surface};
use gb::{gl, glsl};

use gl::shader;
use gl::buffer::{Static, Draw};
use gl::{Program, Buffer, VertexArray};

type Inputs = glsl::Inputs! {
    layout(location = 0) vec3;
    layout(location = 1) vec4;
    layout(location = 2) vec2;
};

type VsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
    layout(location = 1) vec2;
};

type FsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
};

type Uniforms = glsl::Uniforms! {
    layout(location = 0) mat4;
};

// const UNIFORM_VARIABLES: Uniforms = glsl::uniforms! {
//     layout(location = 0) mat4;
// };

type Attributes = hlist::HList! {
    Attribute<[f32; 3], 0>,
    Attribute<[f32; 4], 1>,
    Attribute<[f32; 2], 2>,
};


use crate::Ctx;

pub struct Sample { }

impl crate::Sample for Sample {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        // ========================[ gpu-bulwark ]========================

        let vs_source = std::fs::read_to_string("samples/shaders/basic.vert")?;
        let common_source = std::fs::read_to_string("samples/shaders/basic-common.vert")?;
        let fs_source = std::fs::read_to_string("samples/shaders/basic.frag")?;


        let vs_inputs = Inputs::default();
        let glsl::vars![vin_position, vin_color, vin_tex] = &vs_inputs;

        let vs_outputs = VsOutputs::default();

        let fs_inputs = vs_outputs.matching_inputs();
        let glsl::vars![ fs_output ] = FsOutputs::default();

        let glsl::vars![ view_matrix_location ] = Uniforms::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
        let mut common = shader::create::<shader::target::Vertex>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);
        common.source(&[&common_source]);
    
        let vs = uncompiled_vs
            .uniform(&view_matrix_location)
            .compile()?
            .into_main()
            .inputs(&vs_inputs)
            .outputs(&vs_outputs);
        let fs = uncompiled_fs
            .compile()?
            .into_main()
            .inputs(&fs_inputs)
            .output(&fs_output);
        let common = common.compile()?.into_shared();
    
        let program = Program::builder()
            .uniform_definitions(|definitions| definitions
                .define(&view_matrix_location, &[[0f32; 4]; 4])
            )
            .vertex_main(&vs)
            .uniforms(|matcher| matcher.bind(&view_matrix_location))
            .vertex_shared(&common)
            .fragment_main(&fs)
            .build()?;
    
        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(&[[-0.5, -0.5, -1.0], [0.5, -0.5, -1.0], [0.0, 0.5, -1.0f32]]);
    
        let mut colors = Buffer::create();
        colors.data::<(Static, Draw)>(&[
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]);
    
        let mut texture_coords = Buffer::create();
        texture_coords.data::<(Static, Draw)>(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0f32]]);
    
        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_color, colors)
            .vertex_attrib_pointer(&vin_tex, texture_coords);

        Ok(Ctx {
            window,
            surface,
            context,
            inner: Self {

            },
        })
    }
    
    fn render(&mut self) {
        todo!()
    }
    
    fn process_key(&mut self, code: winit::keyboard::KeyCode) {
        todo!()
    }
    
    fn process_mouse(&mut self, delta: (f64, f64)) {
        todo!()
    }
}