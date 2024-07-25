#![allow(unused_unsafe)]

use std::num::NonZeroU32;

use anyhow;
use glutin::{config, context, display, surface};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::{
    dpi,
    event::{ElementState, Event, RawKeyEvent, WindowEvent},
    event_loop,
    keyboard::PhysicalKey,
    window,
};

use config::ConfigTemplateBuilder;
use context::{ContextApi, ContextAttributesBuilder, GlProfile, Version};
use display::DisplayApiPreference;
use event_loop::EventLoop;
use glutin::prelude::*;
use surface::{SurfaceAttributesBuilder, WindowSurface};

mod builder;
mod constraint;
mod error;
mod ext;
pub mod glsl;
pub mod hlist;
pub mod gl;
pub mod prelude;
mod renderer;
mod types;
pub mod ffi;
mod utils;
pub mod md;
pub mod ts;
pub mod valid;

use glsl::prelude::MatchingInputs;
use gl::shader;
use gl::{
    buffer::{Buffer, Draw, Static},
    program::Program,
    vertex_array::VertexArray,
};
use shader::target::{Fragment, Vertex};
use shader::Shader;

use nalgebra_glm as glm;

fn main() -> anyhow::Result<()> {
    println!("opening event loop...");
    let event_loop = EventLoop::new().expect("window creation is possible");

    let width = 960;
    let height = 640;

    println!("creating window...");
    let window = window::Window::bu
        .with_inner_size(dpi::PhysicalSize { width, height })
        .with_title("gpu-bulwark")
        .with_resizable(false)
        .build(&event_loop)
        .expect("window creation is successful");

    let version = Version::new(4, 6);
    println!(
        "initializing OpenGL {}.{} core",
        version.major, version.minor
    );

    let window_handle = window.raw_window_handle();
    let display_handle = window.raw_display_handle();

    let template = ConfigTemplateBuilder::new().build();
    let context_attributes = ContextAttributesBuilder::new()
        .with_debug(true)
        .with_context_api(ContextApi::OpenGl(Some(version)))
        .with_profile(GlProfile::Core)
        .build(Some(window_handle));

    let (window_width, window_height) = {
        (
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        )
    };

    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window_handle,
        window_width,
        window_height,
    );

    let preference = DisplayApiPreference::WglThenEgl(Some(window_handle));

    // SAFETY: we just checked if handle is valid? (maybe there are some more cavitates to this)
    let display = unsafe { glutin::display::Display::new(display_handle, preference).unwrap() };

    println!("checking available configurations...");
    let config = unsafe { display.find_configs(template) }?
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
    let gl_context = unsafe { display.create_context(&config, &context_attributes)? };

    println!("creating rendering surface...");
    let surface = unsafe { display.create_window_surface(&config, &surface_attributes)? };

    println!("making context current");
    let gl_context = gl_context.make_current(&surface)?;

    println!("loading function pointers...");
    glb::load_with(|symbol| {
        let symbol = std::ffi::CString::new(symbol).unwrap();
        display.get_proc_address(symbol.as_c_str()).cast()
    });

    println!("setting up rendering state...");

    // ========================[ gpu-bulwark ]========================

    let vs_source = std::fs::read_to_string("samples/shaders/basic.vert")?;
    let common_source = std::fs::read_to_string("samples/shaders/basic-common.vert")?;
    let fs_source = std::fs::read_to_string("samples/shaders/basic.frag")?;

    let uncompiled_vs = Shader::<Vertex>::create();
    let uncompiled_fs = Shader::<Fragment>::create();
    let common = Shader::<Vertex>::create();

    uncompiled_vs.source(&[&vs_source]);
    uncompiled_fs.source(&[&fs_source]);
    common.source(&[&common_source]);

    let unpack![view_matrix_location] = uniforms! {
        layout(location = 0) mat4;
    };

    let vs_inputs = inputs! {
        layout(location = 0) vec3;
        layout(location = 1) vec4;
        layout(location = 2) vec2;
    };

    let vs_outputs = outputs! {
        layout(location = 0) vec4;
        layout(location = 1) vec2;
    };

    let fs_inputs = vs_outputs.matching_inputs();

    let ((), fs_outputs) = outputs! {
        layout(location = 0) vec4;
    };

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
        .output(&fs_outputs);
    let common = common.compile()?.into_shared();

    let mut scale = 0f32;
    let mut camera =
        camera::Camera::new(glm::Vec3::zeros(), 0.0, 0.0, width as f32 / height as f32);

    let mut program = Program::builder()
        .uniforms(|definitions| {
            definitions.define(&view_matrix_location, camera.view_projection_matrix())
        })
        .vertex_main(&vs)
        .bind_uniforms(|declarations| declarations.bind(&view_matrix_location))
        .vertex_shared(&common)
        .fragment_main(&fs)
        .build()?;

    let mut positions = Buffer::create();
    positions.data::<(Static, Draw)>(&[[-0.5, -0.5, -1.0], [0.5, -0.5, -1.0], [0.0, 0.5, -1.0]]);

    let mut colors = Buffer::create();
    colors.data::<(Static, Draw)>(&[
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
    ]);

    let mut texture_coords = Buffer::create();
    texture_coords.data::<(Static, Draw)>(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0f32]]);

    let vao = VertexArray::create()
        .attach::<0, _>(&positions)
        .attach::<1, _>(&colors)
        .attach::<2, _>(&texture_coords);

    println!("running main loop...");

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

            let width = width as usize;
            let height = height as usize;

            let mut texture_test = Vec::<[u8; 3]>::with_capacity(width * height);
            for i in 0..(width * height) {
                let signed = i as i64 ;
                texture_test.push([(signed % 256) as _, ((signed - 64) % 256) as _, (signed % 128) as _]);
            }

            // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_WRAP_S, glb::REPEAT as _);
            // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_WRAP_T, glb::REPEAT as _);
            // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_MIN_FILTER, glb::LINEAR_MIPMAP_LINEAR as _);
            // glb::TexParameteri(glb::TEXTURE_2D, glb::TEXTURE_MAG_FILTER, glb::LINEAR as _);

            glb::TexImage2D(
                glb::TEXTURE_2D,
                0,
                glb::RGB as _,
                width as _,
                height as _,
                0,
                glb::RGB,
                glb::UNSIGNED_BYTE,
                texture_test.as_ptr() as *const _
            );
            glb::GenerateMipmap(glb::TEXTURE_2D);
        }
    }

    event_loop.run(move |event, window_target| {
        match event {
            Event::Suspended => {
                // todo read about context yielding on different platforms.
                println!("suspended");
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(size) => {
                    println!("resizing...");
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        surface.resize(
                            &gl_context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }
                }
                WindowEvent::CloseRequested => window_target.exit(),
                WindowEvent::RedrawRequested => {
                    scale += if scale > 1.0 { -1.0 } else { 0.01 };

                    gl::draw_arrays(&vao, &program);

                    surface
                        .swap_buffers(&gl_context)
                        .expect("buffer swapping is successful");
                    window.request_redraw();
                    unsafe {
                        glb::Clear(glb::COLOR_BUFFER_BIT);
                    }
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                    camera.process_mouse(dx, dy);
                    println!("camera yaw {}, pitch {}", camera.yaw, camera.pitch);
                    program.uniform(&view_matrix_location, &camera.view_projection_matrix());
                }
                winit::event::DeviceEvent::Key(RawKeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: ElementState::Pressed,
                }) => {
                    println!("camera at {}", camera.position);
                    camera.process_input(&code);
                }
                _ => (),
            },
            _ => (),
        }
    })?;

    Ok(())
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
                winit::keyboard::KeyCode::KeyW => Some(Vec3::new(0.0, 0.0, -1.0)),
                winit::keyboard::KeyCode::KeyS => Some(Vec3::new(0.0, 0.0, 1.0)),
                winit::keyboard::KeyCode::KeyA => Some(Vec3::new(-1.0, 0.0, 0.0)),
                winit::keyboard::KeyCode::KeyD => Some(Vec3::new(1.0, 0.0, 0.0)),
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
