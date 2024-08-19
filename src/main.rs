#![allow(unused_unsafe)]

use std::cell::LazyCell;
use std::num::NonZeroU32;

mod builder;
mod constraint;
mod ext;
pub mod glsl;
pub mod hlist;
pub mod gl;
pub mod prelude;
mod renderer;
pub mod ffi;
mod utils;
pub mod md;
pub mod ts;
pub mod valid;


use gl::shader;
use gl::{
    buffer::{Buffer, Draw, Static},
    program::Program,
    vertex_array::{VertexArray, Attribute},
};
use glsl::{MatchingInputs, UniformVariable};

use nalgebra_glm as glm;

use winit::application::ApplicationHandler;
use winit::event::{self, DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{self, ActiveEventLoop, EventLoop};
use winit::keyboard;
use winit::window::{self, Window};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle as _};

use glutin::{context, display, surface};
use glutin::prelude::*;


thread_local! {
    static CONTEXT: std::cell::OnceCell<Context> = std::cell::OnceCell::new();
}

/// FIXME: This is a temporary solution to the question of context handling.
pub struct Context {
    inner: glutin::context::PossiblyCurrentContext,
}

// impl Context {
//     pub fn global(&self) {
//         CONTEXT.with(|once_cell| {
//             once_cell.get_or_init(|inner| );
//         });
//     }
// }

fn main() -> anyhow::Result<()> {
    
}

mod camera {
    use glm::Vec3;
    use nalgebra_glm as glm;

    const MOVEMENT_SPEED: f32 = 0.1;
    const MOUSE_SENSITIVITY: f32 = 0.005;

    pub struct Camera {
        pub position: Vec3,
        pub yaw: f32,
        pub pitch: f32,
        aspect_ratio: f32,
    }

    impl Camera {
        pub fn new(position: Vec3, yaw: f32, pitch: f32, aspect_ratio: f32) -> Self {
            Camera {
                position,
                yaw,
                pitch,
                aspect_ratio,
            }
        }

        fn view_matrix(&self) -> glm::Mat4 {
            let front = Vec3::new(
                self.yaw.cos() * self.pitch.cos(),
                self.pitch.sin(),
                self.yaw.sin() * self.pitch.cos(),
            );
            glm::look_at_rh(&self.position, &(self.position + front), &Vec3::y())
        }

        fn projection_matrix(&self) -> glm::Mat4 {
            glm::perspective_rh_no(
                45.0 * std::f32::consts::FRAC_1_PI * 0.5,
                self.aspect_ratio,
                0.1,
                1000.0,
            )
        }

        pub fn view_projection_matrix(&self) -> glm::Mat4 {
            self.projection_matrix() * self.view_matrix()
        }

        pub fn process_input(&mut self, input: &winit::keyboard::KeyCode) {
            match input {
                winit::keyboard::KeyCode::KeyW => Some(Vec3::new( 0.0,  0.0, -1.0)),
                winit::keyboard::KeyCode::KeyS => Some(Vec3::new( 0.0,  0.0,  1.0)),
                winit::keyboard::KeyCode::KeyA => Some(Vec3::new(-1.0,  0.0,  0.0)),
                winit::keyboard::KeyCode::KeyD => Some(Vec3::new( 1.0,  0.0,  0.0)),
                _ => None,
            }
            .inspect(|movement| {
                let rotation_matrix = glm::rotation(self.yaw, &Vec3::y());
                self.position += (rotation_matrix
                    * glm::vec4(movement.x, movement.y, movement.z, 1.0)
                    * MOVEMENT_SPEED)
                    .xyz();
            });
        }

        pub fn process_mouse(&mut self, dx: f64, dy: f64) {
            self.yaw += dx as f32 * MOUSE_SENSITIVITY;
            self.pitch += dy as f32 * MOUSE_SENSITIVITY;

            if self.pitch > 1.5 {
                self.pitch = 1.5;
            }
            if self.pitch < -1.5 {
                self.pitch = -1.5;
            }
        }
    }
}

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

struct App {
    graphics_context: Option<GraphicsContext>,
    window: Option<Window>,
    surface: Option<surface::Surface<surface::WindowSurface>>,
    scale: f32,
    camera: camera::Camera,
    context: context::PossiblyCurrentContext,
    program: gl::Program<Inputs, FsOutputs, Uniforms>,
    vao: VertexArray<Attributes>,
}

pub mod config {
    pub const WIDTH: u32 = 960;
    pub const HEIGHT: u32 = 640;
}

pub struct GraphicsContext {
    window: window::Window,
    surface: surface::Surface<surface::WindowSurface>,
    context: context::PossiblyCurrentContext,
}

impl App {
    pub fn initialize() -> anyhow::Result<Self> {
        // ========================[ gpu-bulwark ]========================

        let vs_source = std::fs::read_to_string("samples/shaders/basic.vert")?;
        let common_source = std::fs::read_to_string("samples/shaders/basic-common.vert")?;
        let fs_source = std::fs::read_to_string("samples/shaders/basic.frag")?;


        let vs_inputs = Inputs::default();
        let vars![vin_position, vin_color, vin_tex] = &vs_inputs;

        let vs_outputs = VsOutputs::default();

        let fs_inputs = vs_outputs.matching_inputs();
        let vars![ fs_output ] = FsOutputs::default();

        let vars![ view_matrix_location ] = Uniforms::default();

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
    
        let mut scale = 0f32;
    
        let mut program = Program::builder()
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

        unsafe {
            glb::ClearColor(0.29, 0.48, 0.73, 0.5);
            glb::Clear(glb::COLOR_BUFFER_BIT);
        }

        let mut texture = 0;
        gl::call! {
            [panic]
            unsafe {
                glb::ActiveTexture(glb::TEXTURE0 + 8);
                glb::CreateTextures(glb::TEXTURE_2D, 1, &mut texture);

                let width = config::WIDTH as usize;
                let height = config::HEIGHT as usize;

                let mut texture_test = Vec::<[u8; 3]>::with_capacity(width * height);
                for i in 0..(width * height) {
                    let signed = i as i64 ;
                    texture_test.push([(signed % 256) as _, ((signed - 64) % 256) as _, (signed % 128) as _]);
                }

                // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_WRAP_S, glb::REPEAT as _);
                // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_WRAP_T, glb::REPEAT as _);
                // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_MIN_FILTER, glb::LINEAR_MIPMAP_LINEAR as _);
                // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_MAG_FILTER, glb::LINEAR as _);

                glb::TexStorage2D(
                    
                );
                glb::GenerateMipmap(glb::TEXTURE_2D);
            }
        }

        Ok(Self {
            graphics_context: None,
            window: None,
            surface: None,
            scale: 1.0,
            camera: camera::Camera::new(glm::Vec3::zeros(), 0.0, 0.0, config::WIDTH as f32 / config::HEIGHT as f32),
            program,
            context: todo!(),
            vao: todo!(),
        })
    }

    fn render(&mut self) {
        self.scale += if self.scale > 1.0 { -1.0 } else { 0.01 };

        gl::draw_arrays(&self.vao, &self.program);

        self.surface
            .as_mut()
            .unwrap()
            .swap_buffers(&self.context)
            .expect("buffer swapping is successful");

        self.window.as_ref().unwrap().request_redraw();
        gl::call! {
            [panic]
            unsafe {
                glb::Clear(glb::COLOR_BUFFER_BIT);
            }
        };
    }

    fn process_keyboard_input(&mut self, key: keyboard::KeyCode) {
        println!("camera at {}", self.camera.position);
        self.camera.process_input(&key);
    }

    fn process_mouse_input(&mut self, (dx, dy): (f64, f64)) {
        let camera = &mut self.camera;
        camera.process_mouse(dx, dy);
        println!("camera yaw {}, pitch {}", camera.yaw, camera.pitch);
        self.program.uniform(&VIEW_MATRIX_LOCATION, &camera.view_projection_matrix());
    }
}

const VIEW_MATRIX_LOCATION: UniformVariable<glsl::Mat4, 0> = UniformVariable::new_phantom();


impl ApplicationHandler for App {
    fn suspended(&mut self, _: &ActiveEventLoop) {
        // TODO: read about context yielding on different platforms.
        println!("suspended");
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: event::DeviceId, event: event::DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta } => { self.process_mouse_input(delta) }
            DeviceEvent::Key(winit::event::RawKeyEvent {
                physical_key: keyboard::PhysicalKey::Code(key),
                state: ElementState::Pressed,
            }) => {
                self.process_keyboard_input(key)
            },
            _ => (),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                println!("resizing...");
                if size.width != 0 && size.height != 0 {
                    // Some platforms like EGL require resizing GL surface to update the size
                    // Notable platforms here are Wayland and macOS, other don't require it
                    // and the function is no-op, but it's wise to resize it for portability
                    // reasons.
                    self.surface.as_mut().unwrap().resize(
                        &self.context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                }
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render(),
            _ => (),
        }
    }
    
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        let window = event_loop.create_window(
            winit::window::WindowAttributes::default()
                .with_inner_size(winit::dpi::PhysicalSize::new(config::WIDTH, config::HEIGHT))
                .with_title("gpu-bulwark")
                .with_resizable(false)
        ).expect("window creation succeeded");

        let version = context::Version::new(4, 6);
        println!(
            "initializing OpenGL {}.{} core",
            version.major, version.minor
        );
    
        let raw_window_handle = window.window_handle()
            .expect("can obtain window handle")
            .as_raw();
        let raw_display_handle = window.display_handle()
            .expect("can obtain display handle")
            .as_raw();
    
        let template = glutin::config::ConfigTemplateBuilder::new().build();
        let context_attributes = context::ContextAttributesBuilder::new()
            .with_debug(true)
            .with_context_api(context::ContextApi::OpenGl(Some(version)))
            .with_profile(context::GlProfile::Core)
            .build(Some(raw_window_handle));
    
        let (window_width, window_height) = {
            (
                NonZeroU32::new(config::WIDTH).unwrap(),
                NonZeroU32::new(config::HEIGHT).unwrap(),
            )
        };
    
        let surface_attributes = surface::SurfaceAttributesBuilder::<surface::WindowSurface>::new().build(
            raw_window_handle,
            window_width,
            window_height,
        );
    
        let preference = display::DisplayApiPreference::WglThenEgl(Some(raw_window_handle));
    
        // SAFETY: we just checked if handle is valid? (maybe there are some more cavitates to this)
        let display = unsafe { glutin::display::Display::new(raw_display_handle, preference).unwrap() };
    
        println!("checking available configurations...");
        let config = unsafe { display.find_configs(template) }.expect("can find matching configurations")
            .reduce(|accum, config| {
                let transparency_check = config.supports_transparency().unwrap_or(false)
                    & !accum.supports_transparency().unwrap_or(false);
    
                if transparency_check || config.num_samples() > accum.num_samples() {
                    config
                } else {
                    accum
                }
            })
            .expect("at least one configuration is compatible with given template");
    
        println!("using config:");
        println!(
            "  color attachment: {:?}",
            config
                .color_buffer_type()
                .expect("selected config contains color attachment")
        );
        println!("  alpha bits: {}", config.alpha_size());
        println!("  hardware acceleration: {}", config.hardware_accelerated());
        println!("  sample count: {}", config.num_samples());
    
        println!("creating context...");
        let gl_context = unsafe { display.create_context(&config, &context_attributes).expect("can create GL context") };
    
        println!("creating rendering surface...");
        let surface = unsafe { display.create_window_surface(&config, &surface_attributes).expect("can create window surface") };
    
        println!("making context current");
        let gl_context = gl_context.make_current(&surface).expect("can make surface current");
    
        println!("loading function pointers...");
        glb::load_with(|symbol| {
            let symbol = std::ffi::CString::new(symbol).unwrap();
            display.get_proc_address(symbol.as_c_str()).cast()
        });
    
        self.surface = None;
    }
}
