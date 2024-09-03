
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
    layout(location = 1) in vec4;
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

type Uniforms = glsl::Glsl! {
    layout(location = 0) uniform float;
    layout(location = 1) uniform float;
    layout(location = 2) uniform float;
};

type Attributes = gb::HList! {
    Attribute<[f32; 3], 0>,
    Attribute<[f32; 4], 1>,
};

pub struct Listing {
    program: Program<VsInputs, FsOutputs, Uniforms, ()>,
    vao: VertexArray<Attributes>,
    attenuation: f32,
    x_offset: f32,
    y_offset: f32,
    color_shift: f32,
}

impl crate::Sample for Listing {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        let vs_source = std::fs::read_to_string("shaders/vert.glsl")?;
        let fs_source = std::fs::read_to_string("shaders/frag.glsl")?;

        let vs_inputs  = VsInputs::default();
        let vs_outputs = VsOutputs::default();
        let fs_inputs  = FsInputs::default();
        let fs_outputs = FsOutputs::default();

        let glsl::vars![ fs_output ] = fs_outputs;
        let glsl::vars![ vin_position, vin_color ] = &vs_inputs;
        let glsl::vars![ location_attenuation, location_x_offset, location_y_offset ] = Uniforms::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);

        let vs = uncompiled_vs
            .uniform(&location_attenuation)
            .uniform(&location_x_offset)
            .uniform(&location_y_offset)
            .compile()?
            .into_main()
            .inputs(&vs_inputs)
            .outputs(&vs_outputs);
        let fs = uncompiled_fs
            .compile()?
            .into_main()
            .inputs(&fs_inputs)
            .output(&fs_output);

        let attenuation = 1.0;
        let x_offset = 0.0;
        let y_offset = 0.0;

        let program = Program::builder()
            .uniforms(|defs| defs
                .define(&location_attenuation, &attenuation)
                .define(&location_x_offset, &x_offset)
                // .define(&location_y_offset, &y_offset)
            )
            .no_resources()
            .vertex_main(&vs)
            .uniforms(|matcher| matcher
                // .bind(&location_y_offset)
                .bind(&location_x_offset)
                .bind(&location_attenuation)
            )
            .fragment_main(&fs)
            .build()?;
    
        let mut positions = Buffer::create();
        let mut colors = Buffer::create();

        positions.data::<(Dynamic, Draw)>(&[
            [-0.5, -0.5, -1.0f32],
            [ 0.5, -0.5, -1.0f32],
            [ 0.0,  0.5, -1.0f32]
        ]);
        colors.data::<(Dynamic, Draw)>(&[
            [1.0, 0.0, 0.0, 1.0f32], 
            [0.0, 1.0, 0.0, 1.0f32], 
            [0.0, 0.0, 1.0, 1.0f32]
        ]);
    
        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_color, colors)
            ;

        let inner = Self {
            program,
            vao,
            attenuation,
            x_offset,
            y_offset,
            color_shift: -1.0,
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
        
        let glsl::vars![ attenuation, x_offset, y_offset ] = Uniforms::default();
        
        if self.attenuation > 1.0 || self.attenuation < 0.0 { 
            self.color_shift *= -1.0;
        }
        self.attenuation = self.attenuation + self.color_shift * 0.005;
        self.x_offset = if self.x_offset < 1.0 { self.x_offset + 0.005 } else { -1.0 };
        self.y_offset = if self.y_offset < 1.0 { self.y_offset + 0.005 } else { -1.0 };

        self.program.uniform(&attenuation, &self.attenuation);
        self.program.uniform(&x_offset, &self.x_offset);
        self.program.uniform(&y_offset, &self.y_offset);
        self.program.draw_arrays(&self.vao);
    }
    
    fn process_key(&mut self, _: winit::keyboard::KeyCode) { }
    
    fn process_mouse(&mut self, _: (f64, f64)) { }
    
    fn usage(&self) -> String {
        String::from("use A, S, D keys to change values of color vertex attribute components")
    }
    
    fn name() -> String {
        String::from("hello-vertices")
    }
}