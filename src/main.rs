#![allow(unused_unsafe)]

use std::num::NonZeroU32;

use anyhow;
use gl;
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
use window::WindowBuilder;

mod error;
mod glsl;
mod object;
mod prelude;
mod target;
mod types;
mod hlist;
mod builder;

use object::{buffer::{Draw, Static, Buffer}, program::Program, vertex_array::VertexArray};
use object::shader;
use shader::Shader;
use glsl::prelude::MatchingInputs;
use shader::target::{Fragment, Vertex};

fn main() -> anyhow::Result<()> {
    println!("opening event loop...");
    let event_loop = EventLoop::new().expect("window creation is possible");

    let width = 960;
    let height = 640;

    println!("creating window...");
    let window = WindowBuilder::new()
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

    let (width, height) = {
        (
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        )
    };

    let surface_attributes = SurfaceAttributesBuilder::<WindowSurface>::new().build(window_handle, width, height);

    let preference = DisplayApiPreference::WglThenEgl(Some(window_handle));

    // SAEFTY: we just checked if handle is valid? (maybe there are some more caviotes to this)
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
    println!("  color attachment: {:?}", config.color_buffer_type().expect("selected config contains color attachment"));
    println!("  alpha bits: {}", config.alpha_size());
    println!("  hardware acceleration: {}", config.hardware_accelerated());
    println!("  sample count: {}", config.num_samples());

    println!("creating context...");
    let gl_context = unsafe { display.create_context(&config, &context_attributes)? };

    println!("creating rendering surface...");
    let surface = unsafe { display.create_window_surface(&config, &surface_attributes)? };

    println!("making context current");
    let gl_context = gl_context
        .make_current(&surface)?;

    println!("loading function pointers...");
    gl::load_with(|symbol| {
        let symbol = std::ffi::CString::new(symbol).unwrap();
        display.get_proc_address(symbol.as_c_str()).cast()
    });

    println!("setting up rednering state...");
    

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

    // let ((((((), aspect_ratio_location), scale_location), position_location), mvp_location), _)
    let locations![
        aspect_ratio_location, scale_location, position_location, mvp_location, _
    ] = uniforms! {
        layout(location = 0) f32;
        layout(location = 1) glsl::Vec3;
        layout(location = 2) glsl::Vec2;
        layout(location = 3) glsl::Mat4;
        layout(location = 7) f32;
    };

    let vs_inputs = inputs! {
        layout(location = 0) glsl::Vec3;
        layout(location = 1) glsl::Vec4;
    };

    let vs_outputs = outputs! {
        layout(location = 0) glsl::Vec4;
    };

    let fs_inputs = vs_outputs.matching_intputs();
    
    let ((), fs_outputs) = outputs! {
        layout(location = 0) glsl::Vec4;
    };

    let vs = uncompiled_vs
        .uniform(&aspect_ratio_location)
        .uniform(&scale_location)
        .uniform(&mvp_location)
        .compile()?
        .into_main()
        .inputs(&vs_inputs)
        .outputs(&vs_outputs);
    let fs = uncompiled_fs
        .uniform(&position_location)
        .compile()?
        .into_main()
        .inputs(&fs_inputs)
        .output(&fs_outputs);
    let common = common
        .compile()?
        .into_shared();

    let mut aspect_ratio = 0f32;
    let scale = [1.0f32, 1.0, 1.0];
    let positions = [0.; 2];
    let mvp = <[f32; 16]>::default();

    let mut program = Program::builder()
        .uniforms(|definitions| definitions
            .define(&aspect_ratio_location, aspect_ratio)
            .define(&scale_location, scale)
            .define(&position_location, positions)
            .define(&mvp_location, mvp)
        )
        .vertex_main(&vs)
        .bind_uniforms(|declarations| declarations
            .bind(&aspect_ratio_location)
            .bind(&scale_location)
            .bind(&mvp_location)
        )
        .vertex_shared(&common)
        .fragment_main(&fs)
        .bind_uniforms(|declarations| declarations
            .bind(&position_location)
        )
        .build()?;

    program.uniform(&aspect_ratio_location, &2.0);

    let mut positions = Buffer::create();
    positions.data::<(Static, Draw)>(&[[-0.5, -0.5, -1.0], [0.5, -0.5, -1.0], [0.0, 0.5, -1.0]]);

    let mut colors = Buffer::create();
    colors.data::<(Static, Draw)>(&[[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]);

    let vao = VertexArray::create()
        .attach::<0, _>(&positions)
        .attach::<1, _>(&colors);

    println!("running main loop...");

    unsafe {
        gl::ClearColor(0.29, 0.48, 0.73, 0.5);
        gl::Clear(gl::COLOR_BUFFER_BIT);
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
                    aspect_ratio += if aspect_ratio > 1.0 { -1.0 } else { 0.01 };
            
                    program.uniform(&aspect_ratio_location, &aspect_ratio);
            
                    object::draw_arrays(&vao, &program);
            
                    surface
                        .swap_buffers(&gl_context)
                        .expect("buffer swapping is successful");
                    window.request_redraw();
                }
                _ => (),
            },
            Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::Key(RawKeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: ElementState::Pressed,
                }) => {
                    println!("pressed {:?}", code);
                }
                _ => (),
            },
            _ => (),
        }
    })?;

    Ok(())
}
