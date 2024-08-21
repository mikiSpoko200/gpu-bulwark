//! Application that can load and execute samples. Sample to run is selected using features.

mod common;
mod hello_textures;
mod hello_triangle;
mod hello_uniforms;
mod hello_vertices;

use std::num::NonZeroU32;

use winit::application::ApplicationHandler;
use winit::event::{self, DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{self, ActiveEventLoop, EventLoop};
use winit::keyboard;
use winit::window::{self};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle as _};

use common::config;

use glutin::{context, display, surface};
use glutin::prelude::*;

use gb::{gl, glsl};

pub trait Sample: Sized {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>>;

    fn render(&mut self);

    fn process_key(&mut self, code: winit::keyboard::KeyCode);
    
    fn process_mouse(&mut self, delta: (f64, f64));
}

pub struct Ctx<T> {
    window: window::Window,
    surface: surface::Surface<surface::WindowSurface>,
    context: context::PossiblyCurrentContext,
    inner: T,
}

impl<T> AsRef<T> for Ctx<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsMut<T> for Ctx<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[derive(Default)]
struct App<T: Sample> {
    ctx: Option<Ctx<T>>,
}

impl App<hello_triangle::Sample> {
    fn hello_triangle() -> Self {
        todo!()
    }
}

impl App<hello_vertices::Sample> {
    fn hello_vertices() -> Self {
        todo!()
    }
}
impl App<hello_uniforms::Sample> {
    fn hello_uniforms() -> Self {
        todo!()
    }
}

impl App<hello_textures::Sample> {
    fn hello_textures() -> App<hello_textures::Sample> {
        todo!()
    }
}



impl<T: Sample> App<T> {
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.ctx.is_none() {
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
        
            // SAFETY: `raw_display_handle` will remain valid on PC
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
            // SAFETY: display handle is valid
            let gl_context = unsafe { display.create_context(&config, &context_attributes).expect("can create GL context") };
        
            println!("creating rendering surface...");
            // SAFETY: display handle is valid
            let surface = unsafe { display.create_window_surface(&config, &surface_attributes).expect("can create window surface") };
        
            println!("making context current");
            let gl_context = gl_context.make_current(&surface).expect("can make surface current");
        
            println!("loading function pointers...");
            gb::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                display.get_proc_address(symbol.as_c_str()).cast()
            });
            self.ctx = Some(T::initialize(window, surface, gl_context));
        }
    }

    fn render(&mut self) {
        self.ctx.as_mut().map(|ctx| {
            ctx.inner.render();
            
            ctx.surface
                .swap_buffers(&ctx.context)
                .expect("buffer swapping is successful");
    
            ctx.window.request_redraw();
        });
    }

    fn process_key(&mut self, key: keyboard::KeyCode) {
        self.ctx
            .as_mut()
            .map(AsMut::as_mut)
            .map(|sample| sample.process_key(key));
    }

    fn process_mouse_input(&mut self, delta: (f64, f64)) {
        self.ctx
            .as_mut()
            .map(AsMut::as_mut)
            .map(|sample| sample.process_mouse(delta));
    }
}

const VIEW_MATRIX_LOCATION: glsl::UniformVariable<glsl::Mat4, 0> = glsl::UniformVariable::default();


impl<T: Sample> ApplicationHandler for App<T> {
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
                self.process_key(key)
            },
            _ => (),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(ref mut ctx) = self.ctx {
                    println!("resizing...");
                    if size.width != 0 && size.height != 0 {
                        // Some platforms like EGL require resizing GL surface to update the size
                        // Notable platforms here are Wayland and macOS, other don't require it
                        // and the function is no-op, but it's wise to resize it for portability
                        // reasons.
                        ctx.surface.resize(
                            &ctx.context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }
            }
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => self.render(),
            _ => (),
        }
    }
    
    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        if self.ctx.is_none() {
            self.init(event_loop);
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App::hello_triangle();
    let event_loop = EventLoop::new()?;
    Ok(event_loop.run_app(&mut app)?)
}
