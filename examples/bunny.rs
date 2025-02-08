use common::config::{model_path, shader_path};
use gb::{gl, glsl};
use glutin::{context, surface};
use gpu_bulwark as gb;
use nalgebra_glm as glm;
use winit::event::ElementState;
use winit::window;

use gl::buffer::{Draw, Static};
use gl::shader;
use gl::vertex_array::Attribute;
use gl::{Buffer, Program, VertexArray};
use glsl::MatchingInputs as _;

#[path = "common/common.rs"]
mod common;
use common::camera::{Camera, CameraProvider as _, FixedMovable, FreeRoamingCamera, Rotatable};
use common::{Ctx, KeyBoard};

type Inputs = glsl::Inputs! {
    layout(location = 0) vec3;
    layout(location = 1) vec3;
};

type VsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
};

type FsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
};

type Uniforms = glsl::Uniforms! {
    layout(location = 0) mat4;
    layout(location = 1) vec3;
};

type Attributes = gb::HList! {
    Attribute<glm::Vec3, 0>,
    Attribute<glm::Vec3, 1>,
};

pub struct Sample {
    program: Program<Inputs, FsOutputs, Uniforms, ()>,
    vao: VertexArray<Attributes, u16>,
    global_light_dir: glm::Vec3,
    camera: FreeRoamingCamera,
    keyboard: KeyBoard,
}

struct Model {
    pub positions: Vec<glm::Vec3>,
    pub normals: Vec<glm::Vec3>,
    pub index: Vec<u16>,
}

impl Sample {
    fn load_model() -> Model {
        use obj::load_obj;
        use std::fs::File;
        use std::io::BufReader;

        let input = BufReader::new(File::open(model_path("bunny.obj")).expect("model file exists"));
        let obj = load_obj::<obj::Vertex, _, _>(input).expect("model can be loaded");
        let (positions, normals) = obj
            .vertices
            .iter()
            .map(|vertex| {
                (
                    glm::Vec3::from(vertex.position),
                    glm::Vec3::from(vertex.normal),
                )
            })
            .unzip();
        Model {
            positions,
            normals,
            index: obj.indices,
        }
    }
}

impl common::Sample for Sample {
    fn initialize(
        window: window::Window,
        surface: surface::Surface<surface::WindowSurface>,
        context: context::PossiblyCurrentContext,
    ) -> anyhow::Result<Ctx<Self>> {
        // ========================[ gpu-bulwark ]========================

        let vs_source = std::fs::read_to_string(shader_path("bunny.vert"))?;
        let fs_source = std::fs::read_to_string(shader_path("bunny.frag"))?;

        let vs_inputs = Inputs::default();
        let glsl::vars![vin_position, vin_color] = &vs_inputs;

        let vs_outputs = VsOutputs::default();

        let fs_inputs = vs_outputs.matching_inputs();
        let glsl::vars![fs_output] = FsOutputs::default();

        let glsl::vars![view_matrix_location, global_light_direction_location] =
            Uniforms::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();

        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);

        let camera = FreeRoamingCamera::from(Camera::default());

        let vs = uncompiled_vs
            .uniform(&view_matrix_location)
            .uniform(&global_light_direction_location)
            .compile()?
            .into_main()
            .inputs(&vs_inputs)
            .outputs(&vs_outputs);
        let fs = uncompiled_fs
            .compile()?
            .into_main()
            .inputs(&fs_inputs)
            .output(&fs_output);

        let global_light_dir = glm::vec3(-1f32, -1f32, -1f32);

        let matrix = camera.view_projection_matrix();

        let program = Program::builder()
            .uniforms(|definitions| {
                definitions
                    .define(&view_matrix_location, &matrix)
                    .define(&global_light_direction_location, &global_light_dir)
            })
            .no_resources()
            .vertex_main(&vs)
            .uniforms(|matcher| {
                matcher
                    .bind(&global_light_direction_location)
                    .bind(&view_matrix_location)
            })
            .fragment_main(&fs)
            .build()?;

        let model = Self::load_model();

        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(&model.positions);

        let mut colors = Buffer::create();
        colors.data::<(Static, Draw)>(&model.normals);

        let mut index = Buffer::create();
        index.data::<(Static, Draw)>(&model.index);

        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_color, colors)
            .element_buffer(index);

        let mut inner = Self {
            program,
            vao,
            global_light_dir,
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
        self.program.draw_elements(&self.vao);
    }

    fn process_key(&mut self, code: winit::keyboard::KeyCode, state: winit::event::ElementState) {
        self.keyboard
            .as_mut(code)
            .expect("code is supported")
            .set(state == ElementState::Pressed);

        let glsl::vars![matrix, light] = Uniforms::default();
        match code {
            winit::keyboard::KeyCode::KeyW => self
                .camera
                .fixed_move(&crate::common::camera::Direction::Front),
            winit::keyboard::KeyCode::KeyS => self
                .camera
                .fixed_move(&crate::common::camera::Direction::Back),
            winit::keyboard::KeyCode::KeyA => self
                .camera
                .fixed_move(&crate::common::camera::Direction::Left),
            winit::keyboard::KeyCode::KeyD => self
                .camera
                .fixed_move(&crate::common::camera::Direction::Right),
            _ => (),
        };
        self.program
            .uniform(&matrix, &self.camera.view_projection_matrix());
        if code == winit::keyboard::KeyCode::Space {
            self.global_light_dir =
                (glm::rotate(&glm::Mat4::identity(), 0.1, &glm::Vec3::y_axis())
                    * glm::vec3_to_vec4(&self.global_light_dir))
                .xyz();
            self.program.uniform(&light, &self.global_light_dir);
        }
    }

    fn process_mouse(&mut self, (dx, dy): (f64, f64)) {
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

impl common::InteractiveSample for Sample {
    const FREQUENCY: usize = 120;
    type DCtx = common::KeyBoard;

    fn update(&mut self, dctx: &KeyBoard, _: std::time::Duration) {}
}

fn main() -> anyhow::Result<()> {
    common::run_sample::<Sample>()?;
    Ok(())
}
