
use winit::window;
use glutin::{context, surface};
use gb::{gl, glsl};


use gl::vertex_array::Attribute;
use gl::shader;
use gl::buffer::{Static, Draw};
use gl::{Program, Buffer, VertexArray};
use glsl::MatchingInputs as _;


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


use crate::common::camera::Camera;
use crate::common::config;
use crate::Ctx;

pub struct Sample {
    program: Program<Inputs, FsOutputs, Uniforms, ()>,
    vao: VertexArray<Attributes>,
    scale: f32,
    camera: Camera,
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
        
        let program = Program::builder()
            .uniforms(|definitions| definitions
                .define(&view_matrix_location, &[[0f32; 4]; 4])
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
        positions.data::<(Static, Draw)>(&[[-0.5, -0.5, -1.0], [0.5, -0.5, -1.0], [0.0, 0.5, -1.0f32]]);
    
        let mut colors = Buffer::create();
        colors.data::<(Static, Draw)>(&[
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0, 1.0],
        ]);
    
        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_color, colors);
        
        let inner = Self {
            program,
            vao,
            scale,
            camera: Camera::default(),
        };

        Ok(Ctx {
            window,
            surface,
            context,
            inner,
        })
    }
    
    fn render(&mut self) {
        let glsl::vars![ _matrix, scale_location ] = Uniforms::default();

        self.scale += if self.scale > 1.0 { -1.0 } else { 0.01 };
        self.program.uniform(&scale_location, &self.scale);

        self.program.draw_arrays(&self.vao);
    }
    
    fn process_key(&mut self, code: winit::keyboard::KeyCode) {
        match code {
            winit::keyboard::KeyCode::KeyW => Some(glm::Vec3::new( 0.0,  0.0, -1.0)),
            winit::keyboard::KeyCode::KeyS => Some(glm::Vec3::new( 0.0,  0.0,  1.0)),
            winit::keyboard::KeyCode::KeyA => Some(glm::Vec3::new(-1.0,  0.0,  0.0)),
            winit::keyboard::KeyCode::KeyD => Some(glm::Vec3::new( 1.0,  0.0,  0.0)),
            _ => None,
        }
        .inspect(|movement| {
            let rotation_matrix = glm::rotation(self.camera.yaw, &glm::Vec3::y());
            self.camera.position += (rotation_matrix
                * glm::vec4(movement.x, movement.y, movement.z, 1.0)
                * config::MOVEMENT_SPEED)
                .xyz();
        });
        self.render();
    }

    fn usage(&self) -> String {
        String::from("use W, A, S, D keys to move around and mouse to operate the camera")
    }
    
    fn process_mouse(&mut self, (dx, dy): (f64, f64)) {
        self.camera.yaw += dx as f32 * config::MOUSE_SENSITIVITY;
        self.camera.pitch += dy as f32 * config::MOUSE_SENSITIVITY;

        if self.camera.pitch > 1.5 {
            self.camera.pitch = 1.5;
        }
        if self.camera.pitch < -1.5 {
            self.camera.pitch = -1.5;
        }
        self.render();
    }
}