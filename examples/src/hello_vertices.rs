// Sample application imports
use crate::Ctx;

// Windowing library imports
use winit::window;
use glutin::{context, surface};

// gpu-bulwark imports
use gb::{gl, glsl};
use gl::vertex_array::Attribute;
use gl::shader;
use gl::buffer::{Static, Draw};
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

pub struct Sample {
    program: Program<VsInputs, FsOutputs, (), ()>,
    vao: VertexArray<Attributes>,
}

impl Sample {
    // Color values will be shifted by this much with each key press
    const ATTENUATION_FACTOR: f32 = 0.05;
}

impl crate::Sample for Sample {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        // Read shader source code.
        let vs_source = std::fs::read_to_string("shaders/hello_vertices.vert")?;
        let fs_source = std::fs::read_to_string("shaders/hello_vertices.frag")?;

        // GLSL varaible bindings.
        let vs_inputs  = VsInputs::default();
        let vs_outputs = VsOutputs::default();
        let fs_inputs  = FsInputs::default();
        let fs_outputs = FsOutputs::default();

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
        colors.data::<(Static, Draw)>(&[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0f32]]);

        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(&[[-0.5, -0.5, -1.0], [0.5, -0.5, -1.0], [0.0, 0.5, -1.0f32]]);
    
    
    
        // Vertex attribute array configuration.
        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_color, colors)
            .vertex_attrib_pointer(&vin_position, positions)
            ;

        // NOTE: Uncomment this to provoke a compilation error resulting from swapped attribute indices.
        // let vao = VertexArray::create()
        //     .vertex_attrib_pointer(&vin_position, positions)
        //     .vertex_attrib_pointer(&vin_color, colors)
        //     ;
        
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
        self.program.draw_arrays(&self.vao);
    }
    
    fn process_key(&mut self, code: winit::keyboard::KeyCode) {

        let glsl::vars![color, _pos] = VsInputs::default();
        let mut data = self.vao.buffer_mut(&color).map_mut();
        let mut attenuate = |offset| {
            println!("attenuating color: {}", data[0][offset]);

            for vertex_color in data.iter_mut() {
                vertex_color[offset] += Self::ATTENUATION_FACTOR;
                if vertex_color[offset] > 1.0 {
                    vertex_color[offset] = 0.0;
                }
            }
        };

        match code {
            winit::keyboard::KeyCode::KeyA => attenuate(0),
            winit::keyboard::KeyCode::KeyS => attenuate(1),
            winit::keyboard::KeyCode::KeyD => attenuate(2),
            _ => (),
        }
    }
    
    fn process_mouse(&mut self, _: (f64, f64)) { }
}