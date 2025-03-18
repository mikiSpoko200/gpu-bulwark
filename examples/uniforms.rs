use common::config::shader_path;
use gb::{gl, glsl};
use glutin::{context, surface};
use gpu_bulwark as gb;
use winit::event::ElementState;
use winit::window;
use nalgebra_glm as glm;

use gl::buffer::{Draw, Static};
use gl::shader;
use gl::vertex_array::Attribute;
use gl::{Buffer, Program, VertexArray};
use glsl::MatchingInputs as _;

#[path = "common/common.rs"]
mod common;
use common::camera::{Camera, Directions, Projection, View};
use common::{Ctx, KeyBoard};

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

pub struct Sample {
    program: Program<Inputs, FsOutputs, Uniforms, ()>,
    vao: VertexArray<Attributes>,
    scale: f32,
    camera: Camera,
    keyboard: KeyBoard,
}

impl common::Sample for Sample {
    fn initialize(
        window: window::Window,
        surface: surface::Surface<surface::WindowSurface>,
        context: context::PossiblyCurrentContext,
    ) -> anyhow::Result<Ctx<Self>> {
        
        // ========================[ gpu-bulwark ]========================

        let vs_source = std::fs::read_to_string(shader_path("uniforms.vert"))?;
        let common_source = std::fs::read_to_string(shader_path("uniforms_shared.vert"))?;
        let fs_source = std::fs::read_to_string(shader_path("uniforms.frag"))?;

        let vs_inputs = Inputs::default();
        let glsl::vars![vin_position, vin_color] = &vs_inputs;

        let vs_outputs = VsOutputs::default();

        let fs_inputs = vs_outputs.matching_inputs();
        let glsl::vars![fs_output] = FsOutputs::default();

        let glsl::vars![view_matrix_location, scale_location] = Uniforms::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
        let mut common = shader::create::<shader::target::Vertex>();

        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);
        common.source(&[&common_source]);

        let camera = {
            let view = View::new(Directions::BACK, Directions::UP);
            let projection = Projection::perspective(0.01, 100.0, 16.0 / 9.0, 60.0);

            Camera::stationary(view, projection, glm::Vec3::zeros())
        };

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
            .uniforms(|definitions| {
                definitions
                    .define(&view_matrix_location, &matrix)
                    .define(&scale_location, &scale)
            })
            .no_resources()
            .vertex_main(&vs)
            .uniforms(|matcher| matcher.bind(&scale_location).bind(&view_matrix_location))
            .vertex_shared(&common)
            .fragment_main(&fs)
            .build()?;

        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(&[
            [-0.5, -0.5, 0.0],
            [0.5, -0.5, 0.0],
            [0.0, 0.5, 0.0f32],
        ]);

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
            keyboard: KeyBoard::default(),
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
            #[panic]
            unsafe {
                gl::raw::ClearColor(0.4, 0.5, 0.6, 1.0);
                gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT);
            }
        }
        self.program.draw_arrays(&self.vao);
    }

    fn on_key(&mut self, code: winit::keyboard::KeyCode, state: ElementState) {
        match code {
            winit::keyboard::KeyCode::KeyW => self
                .keyboard
                .key_w
                .set(state.is_pressed()),
            winit::keyboard::KeyCode::KeyS => self
                .keyboard
                .key_s
                .set(state.is_pressed()),
            winit::keyboard::KeyCode::KeyA => self
                .keyboard
                .key_a
                .set(state.is_pressed()),
            winit::keyboard::KeyCode::KeyD => self
                .keyboard
                .key_d
                .set(state.is_pressed()),
            _ => (),
        };

        let glsl::vars![matrix, scale] = Uniforms::default();

        self.program
            .uniform(&matrix, &self.camera.view_projection_matrix());
        if code == winit::keyboard::KeyCode::Space {
            self.scale -= 0.01;
            if self.scale < 0.4 {
                self.scale = 1.5;
            }
            self.program.uniform(&scale, &self.scale);
        }
    }

    fn on_mouse_movement(&mut self, (dx, dy): (f64, f64)) {
        let glsl::vars![matrix, _scale] = Uniforms::default();

        self.camera
            .rotate((-dy as f32).to_radians(), (-dx as f32).to_radians());
        self.program
            .uniform(&matrix, &self.camera.view_projection_matrix());
    }

    fn usage(&self) -> String {
        String::from("use W, A, S, D keys to move around and mouse to operate the camera, hold space bar to modify triangle's size")
    }

    fn name() -> String {
        String::from("hello-uniforms")
    }
}

fn main() -> anyhow::Result<()> {
    common::run_sample::<Sample>()?;
    Ok(())
}
