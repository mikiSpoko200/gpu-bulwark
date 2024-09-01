use std::io::Write;

// Sample application imports
use crate::Ctx;

// Windowing library imports
use winit::window;
use glutin::{context, surface};

// gpu-bulwark imports
use gb::{gl, glsl};
use gl::vertex_array::Attribute;
use gl::shader;
use gl::buffer::{Dynamic, Draw};
use gl::{Program, Buffer, VertexArray};

type VsInputs = glsl::Glsl! {
    layout(location = 0) in vec3;
    layout(location = 1) in vec3;
};

type VsOutputs = glsl::Glsl! {
    layout(location = 0) out vec4;
};

type FsInputs = glsl::Glsl! {
    layout(location = 0) in vec4;
};

type FsOutputs = glsl::Glsl! {
    layout(location = 0) out vec4;
};

type Attributes = gb::HList! {
    Attribute<[f32; 3], 0>,
    Attribute<[f32; 3], 1>,
};

pub struct Listing {
    program: Program<VsInputs, FsOutputs, (), ()>,
    vao: VertexArray<Attributes>,
}

impl Listing {
    // Color values will be shifted by this much with each key press
    const ATTENUATION_FACTOR: f32 = 0.005;
}

impl crate::Sample for Listing {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        let vs_source = std::fs::read_to_string("../shaders/hello_vertices.vert")?;
        let fs_source = std::fs::read_to_string("../shaders/hello_vertices.frag")?;

        let vs_inputs  = VsInputs::default();
        let vs_outputs = VsOutputs::default();
        let fs_inputs  = FsInputs::default();
        let fs_outputs = FsOutputs::default();

        let glsl::vars![ fs_output ] = fs_outputs;
        let glsl::vars![ vin_color, vin_position ] = &vs_inputs;

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);

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

        let program = Program::builder()
            .no_uniforms()
            .no_resources()
            .vertex_main(&vs)
            .fragment_main(&fs)
            .build()?;
    
        let mut colors = Buffer::create();
        let mut positions = Buffer::create();

        colors.data::<(Dynamic, Draw)>(&[
            [1.0, 0.0, 0.0, 1.0], 
            [0.0, 1.0, 0.0, 1.0], 
            [0.0, 0.0, 1.0, 1.0f32]
        ]);
        positions.data::<(Dynamic, Draw)>(&[
            [-0.5, -0.5, -1.0], 
            [ 0.5, -0.5, -1.0], 
            [ 0.0,  0.5, -1.0f32]
        ]);
    
        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_color, colors)
            .vertex_attrib_pointer(&vin_position, positions)
            ;

        let inner = Self {
            program,
            vao,
        };

        Ok(Ctx {
            window,
            surface,
            context,
            inner,
        })
    }
    
    fn render(&mut self) {
        gl::call! {
            [panic]
            unsafe {
                gl::raw::ClearColor(0.4, 0.5, 0.6, 1.0);
                gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT);
            }
        }

        self.program.draw_arrays(&self.vao);
    }
    
    fn process_key(&mut self, code: winit::keyboard::KeyCode) {

        let glsl::vars![color, _pos] = VsInputs::default();
        let mut data = self.vao.buffer_mut(&color).map_mut();
        let mut attenuate = |offset| {
            for vertex_color in data.iter_mut() {
                vertex_color[offset] += Self::ATTENUATION_FACTOR;
                if vertex_color[offset] > 1.0 {
                    vertex_color[offset] = 0.0;
                }
            }

            print!("\rchaning {} channel", match offset { 0 => "R", 1 => "G", 2 => "B", _ => panic!("invalid offset {offset}") });
            std::io::stdout().flush().unwrap();
        };

        match code {
            winit::keyboard::KeyCode::KeyA => attenuate(0),
            winit::keyboard::KeyCode::KeyS => attenuate(1),
            winit::keyboard::KeyCode::KeyD => attenuate(2),
            _ => (),
        }
    }
    
    fn process_mouse(&mut self, _: (f64, f64)) { }
    
    fn usage(&self) -> String {
        String::from("use A, S, D keys to change values of color vertex attribute components")
    }
    
    fn name() -> String {
        String::from("hello-vertices")
    }
}