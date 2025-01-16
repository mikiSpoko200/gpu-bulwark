
use winit::window;
use glutin::{context, surface};
use gb::{gl, glsl};


use gl::vertex_array::Attribute;
use gl::shader;
use gl::buffer::{Static, Draw};
use gl::{Program, Buffer, VertexArray};
use glsl::MatchingInputs as _;

use crate::common::camera::{Camera, CameraProvider as _, FixedMovable, Rotatable};

type Inputs = glsl::Inputs! {
    layout(location = 0) vec3;
    layout(location = 1) vec4;
};

type VsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
};

type FsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
};

type Uniforms = glsl::Uniforms! {
    layout(location = 0) mat4;
    layout(location = 1) float;
};

type Attributes = gb::HList! {
    Attribute<[f32; 3], 0>,
    Attribute<[f32; 4], 1>,
};


use crate::common::camera::FreeRoamingCamera;
use crate::Ctx;

pub struct Sample {
    program: Program<Inputs, FsOutputs, Uniforms, ()>,
    vao: VertexArray<Attributes>,
    scale: f32,
    camera: FreeRoamingCamera,
}

impl crate::Sample for Sample {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        // ========================[ gpu-bulwark ]========================

        let vs_source = std::fs::read_to_string("shaders/hello_uniforms.vert")?;
        let common_source = std::fs::read_to_string("shaders/hello_uniforms_shared.vert")?;
        let fs_source = std::fs::read_to_string("shaders/hello_uniforms.frag")?;

        let vs_inputs = Inputs::default();
        let glsl::vars![vin_position, vin_color] = &vs_inputs;

        let vs_outputs = VsOutputs::default();

        let fs_inputs = vs_outputs.matching_inputs();
        let glsl::vars![ fs_output ] = FsOutputs::default();

        let glsl::vars![ view_matrix_location, scale_location ] = Uniforms::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
        let mut common = shader::create::<shader::target::Vertex>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);
        common.source(&[&common_source]);

        let camera = FreeRoamingCamera::from(Camera::default());
    
        let vs = uncompiled_vs
            .uniform(&view_matrix_location)
            .uniform(&scale_location)
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

        let scale = 1.0;
        
        let matrix = camera.view_projection_matrix();

        let program = Program::builder()
            .uniforms(|definitions| definitions
                .define(&view_matrix_location, &matrix)
                .define(&scale_location, &scale)
            )
            .no_resources()
            .vertex_main(&vs)
            .uniforms(|matcher| matcher
                .bind(&scale_location)
                .bind(&view_matrix_location)
            )
            .vertex_shared(&common)
            .fragment_main(&fs)
            .build()?;
    
        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(&[[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0f32]]);
    
        let mut colors = Buffer::create();
        colors.data::<(Static, Draw)>(&[
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]);
    
        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_color, colors);
        
        let mut inner = Self {
            program,
            vao,
            scale,
            camera,
        };

        inner.render();
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
        let glsl::vars![matrix, scale] = Uniforms::default();
        match code {
            winit::keyboard::KeyCode::KeyW => self.camera.fixed_move(&crate::common::camera::Direction::Front),
            winit::keyboard::KeyCode::KeyS => self.camera.fixed_move(&crate::common::camera::Direction::Back),
            winit::keyboard::KeyCode::KeyA => self.camera.fixed_move(&crate::common::camera::Direction::Left),
            winit::keyboard::KeyCode::KeyD => self.camera.fixed_move(&crate::common::camera::Direction::Right),
            _ => (),
        };
        self.program.uniform(&matrix, &self.camera.view_projection_matrix());
        if code == winit::keyboard::KeyCode::Space {
            self.scale -= 0.01;
            if self.scale < 0.4 {
                self.scale = 1.5;
            }
            self.program.uniform(&scale, &self.scale);
        }
    }
    
    fn process_mouse(&mut self, (dx, dy): (f64, f64)) {
        let glsl::vars![matrix, _scale] = Uniforms::default();

        self.camera.rotate((-dy as f32).to_radians(), (-dx as f32).to_radians());
        self.program.uniform(&matrix, &self.camera.view_projection_matrix());
    }

    fn usage(&self) -> String {
        String::from("use W, A, S, D keys to move around and mouse to operate the camera, hold space bar to modify triangle's size")
    }
    
    fn name() -> String {
        String::from("hello-uniforms")
    }
}