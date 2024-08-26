//! Application that can load and execute samples. Sample to run is selected using features.

// Check compilation features
#[cfg(any(
    all(feature = "hello_triangle", any(feature = "hello_uniforms", feature = "hello_vertices", feature = "hello_textures")),
    all(feature = "hello_uniforms", any(feature = "hello_triangle", feature = "hello_vertices", feature = "hello_textures")),
    all(feature = "hello_vertices", any(feature = "hello_triangle", feature = "hello_uniforms", feature = "hello_textures")),
    all(feature = "hello_textures", any(feature = "hello_triangle", feature = "hello_uniforms", feature = "hello_vertices")),
))]
compile_error!("multiple sample features selected, only a single sample can be built at a time");

#[cfg(not(any(feature = "hello_triangle", feature = "hello_vertices", feature = "hello_uniforms", feature = "hello_textures")))]
compile_error!("no sample selected, add `--feature <sample-name>` when building");

mod common;
mod hello_textures;
mod hello_triangle;
mod hello_uniforms;
mod hello_vertices;

use std::num::NonZeroU32;

use glutin::{context, surface};
use glutin::prelude::*;
use glutin::display::GetGlDisplay;

use glutin_winit::{DisplayBuilder, GlWindow};
use winit::application::ApplicationHandler;
use winit::event::{self, DeviceEvent, ElementState, WindowEvent};
use winit::event_loop::{self, ActiveEventLoop, EventLoop};
use winit::keyboard;
use winit::window::{self};
use raw_window_handle::HasWindowHandle as _;

use common::config;

pub trait Sample: Sized {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>>;

    fn render(&mut self);

    fn process_key(&mut self, code: winit::keyboard::KeyCode);
    
    fn process_mouse(&mut self, delta: (f64, f64));

    fn usage(&self) -> String;

    fn config() -> config::Config {
        config::Config::default()
    }
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

struct App<T: Sample> {
    ctx: Option<Ctx<T>>,
}

impl<T: Sample> Default for App<T> {
    fn default() -> Self {
        Self { ctx: None }
    }
}


impl App<hello_triangle::Sample> {
    #[allow(unused)]
    fn hello_triangle() -> Self {
        Self::default()
    }
}

impl App<hello_vertices::Sample> {
    #[allow(unused)]
    fn hello_vertices() -> Self {
        Self::default()
    }
}
impl App<hello_uniforms::Sample> {
    #[allow(unused)]
    fn hello_uniforms() -> Self {
        Self::default()
    }
}

impl App<hello_textures::Sample> {
    #[allow(unused)]
    fn hello_textures() -> App<hello_textures::Sample> {
        Self::default()
    }
}

impl<T: Sample> App<T> {
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.ctx.is_none() {
            let config = T::config();

            // Winit window creation
            let window_attributes = winit::window::WindowAttributes::default()
                .with_inner_size(winit::dpi::PhysicalSize::new(config.width, config.height))
                .with_title("gpu-bulwark")
                .with_resizable(false);
    
            // Glutin gl context initialization
            let version = context::Version::new(4, 6);
            println!(
                "initializing OpenGL {}.{} core",
                version.major, version.minor
            );
            let template = glutin::config::ConfigTemplateBuilder::new();
            let display_builder = DisplayBuilder::new().with_window_attributes(Some(window_attributes));

            let config_selector = |configs: Box<dyn Iterator<Item = glutin::config::Config> + '_>| {
                configs.reduce(|accum, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !accum.supports_transparency().unwrap_or(false);
        
                    if transparency_check || config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .expect("at least one configuration is compatible with given template")
            };

            let (mut window, config) = display_builder.build(event_loop, template, config_selector).expect("can create display");

            let raw_window_handle = window
                .as_ref()
                .and_then(|window| window.window_handle().map(|handle| handle.as_raw()).ok());
        
            let window = window.take().unwrap();
        
            let display = config.display();
        
            let context_attributes = context::ContextAttributesBuilder::new().build(raw_window_handle);
        
            let not_current_gl_context = unsafe {
                display
                    .create_context(&config, &context_attributes)
                    .expect("failed to create context")
            };
        
            let attrs = window
                .build_surface_attributes(<_>::default())
                .expect("Failed to build surface attributes");
            let surface = unsafe { config.display().create_window_surface(&config, &attrs).unwrap() };
        
            // let context_attributes = context::ContextAttributesBuilder::new()
            //     .with_debug(true)
            //     .with_context_api(context::ContextApi::OpenGl(Some(version)))
            //     .with_profile(context::GlProfile::Core)
            //     .build(Some(raw_window_handle));
        
            let gl_context = not_current_gl_context.make_current(&surface).expect("can make surface current");
        
            gb::load_with(|symbol| {
                let symbol = std::ffi::CString::new(symbol).unwrap();
                display.get_proc_address(symbol.as_c_str()).cast()
            });
            self.ctx = Some(match T::initialize(window, surface, gl_context) {
                Ok(ctx) => ctx,
                Err(err) => panic!("{err}"),
            });

            println!("*-----------------------------------------------------------*\n");
            println!("{}", self.ctx.as_ref().unwrap().inner.usage());
            println!("\n*-----------------------------------------------------------*");
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


impl<T: Sample> ApplicationHandler for App<T> {
    fn suspended(&mut self, _: &ActiveEventLoop) { }

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

    fn window_event(&mut self, _: &ActiveEventLoop, _: window::WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(ref mut ctx) = self.ctx {
                    println!("resizing...");
                    if size.width != 0 && size.height != 0 {
                        ctx.surface.resize(
                            &ctx.context,
                            NonZeroU32::new(size.width).unwrap(),
                            NonZeroU32::new(size.height).unwrap(),
                        );
                    }
            }
            },
            WindowEvent::CloseRequested => {
                println!("exitting");
                std::process::exit(0);
            },
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
    
    #[cfg(feature = "hello_triangle")]
    let mut app = App::hello_triangle();
    #[cfg(feature = "hello_vertices")]
    let mut app = App::hello_vertices();
    #[cfg(feature = "hello_uniforms")]
    let mut app = App::hello_uniforms();
    #[cfg(feature = "hello_textures")]
    let mut app = App::hello_textures();

    let event_loop = EventLoop::new()?;
    Ok(event_loop.run_app(&mut app)?)
}
