#![allow(unused)]

use std::cell::{Cell, LazyCell};
use std::num::NonZeroU32;
use std::time::{self, Duration};

use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::{context, surface};

use glutin_winit::{DisplayBuilder, GlWindow};
use raw_window_handle::HasWindowHandle as _;
use winit::application::ApplicationHandler;
use winit::event::{self, DeviceEvent, ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{self, ActiveEventLoop, EventLoop};
use winit::keyboard::{self, KeyCode, PhysicalKey};
use winit::window::{self, CursorGrabMode};

use gpu_bulwark as gb;
use nalgebra_glm as glm;

#[path = "physics.rs"]
mod physics;

#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum Toggle {
    #[default]
    Off = 0,
    On = 1,
}

impl Toggle {
    pub const fn flip(&mut self) -> Self {
        match self {
            Toggle::Off => *self = Self::On,
            Toggle::On => *self = Self::Off,
        };
        *self
    }

    pub fn set(&mut self, state: bool) {
        match state {
            true => *self = Self::On,
            false => *self = Self::Off,
        }
    }

    pub const fn is_on(&self) -> bool {
        matches! { self, Self::On }
    }

    pub const fn is_off(&self) -> bool {
        matches! { self, Self::On }
    }

    pub fn map_on<T>(&self, f: impl FnOnce() -> T) -> Option<T> {
        (matches! { self, Self::On }).then(f)
    }

    pub fn map_off<T>(&self, f: impl FnOnce() -> T) -> Option<T> {
        (matches! { self, Self::Off }).then(f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct KeyBoard {
    pub key_w: Toggle,
    pub key_s: Toggle,
    pub key_a: Toggle,
    pub key_d: Toggle,
    pub key_q: Toggle,
    pub key_e: Toggle,
}

impl KeyBoard {
    pub fn as_ref(&self, key: KeyCode) -> Option<&Toggle> {
        match key {
            KeyCode::KeyW => Some(&self.key_w),
            KeyCode::KeyS => Some(&self.key_s),
            KeyCode::KeyA => Some(&self.key_a),
            KeyCode::KeyD => Some(&self.key_d),
            KeyCode::KeyE => Some(&self.key_e),
            KeyCode::KeyQ => Some(&self.key_q),
            _ => None,
        }
    }

    pub fn as_mut(&mut self, key: KeyCode) -> Option<&mut Toggle> {
        match key {
            KeyCode::KeyW => Some(&mut self.key_w),
            KeyCode::KeyS => Some(&mut self.key_s),
            KeyCode::KeyA => Some(&mut self.key_a),
            KeyCode::KeyD => Some(&mut self.key_d),
            KeyCode::KeyE => Some(&mut self.key_e),
            KeyCode::KeyQ => Some(&mut self.key_q),
            _ => None,
        }
    }
}

pub mod config {
    use std::path::PathBuf;

    pub struct Config {
        pub width: u32,
        pub height: u32,
    }

    impl Default for Config {
        fn default() -> Self {
            Self {
                width: WIDTH,
                height: HEIGHT,
            }
        }
    }

    pub const WIDTH: u32 = 960;
    pub const HEIGHT: u32 = 640;
    pub const MOUSE_SENSITIVITY: f32 = 0.0005;
    pub const MOVEMENT_SPEED: f32 = 0.1;

    const RESOURCE_PATH: &'static str = "examples/resources";
    const SHADER_PATH: &'static str = "examples/shaders";
    const MODEL_PATH: &'static str = "examples/models";

    pub fn resource_path(resource: &'static str) -> PathBuf {
        format!("{}/{}", RESOURCE_PATH, resource).into()
    }

    pub fn shader_path(shader: &'static str) -> PathBuf {
        format!("{}/{}", SHADER_PATH, shader).into()
    }

    pub fn model_path(model: &'static str) -> PathBuf {
        format!("{}/{}", MODEL_PATH, model).into()
    }
}

pub mod camera {
    use super::config::{HEIGHT, WIDTH};
    use super::gb;
    use super::glm::{self, Mat4, Vec3};

    #[derive(Debug, Copy, Clone)]
    pub struct Directions {
        pub up: glm::Vec3,
        pub down: glm::Vec3,
        pub front: glm::Vec3,
        pub back: glm::Vec3,
        pub left: glm::Vec3,
        pub right: glm::Vec3,
    }

    impl Directions {
        const FRONT: glm::Vec3 = glm::Vec3::new(0f32, 0f32, -1f32);
        const BACK: glm::Vec3 = glm::Vec3::new(0f32, 0f32, 1f32);
        const UP: glm::Vec3 = glm::Vec3::new(0f32, 1f32, 0f32);
        const DOWN: glm::Vec3 = glm::Vec3::new(0f32, -1f32, 0f32);
        const RIGHT: glm::Vec3 = glm::Vec3::new(1f32, 0f32, 0f32);
        const LEFT: glm::Vec3 = glm::Vec3::new(-1f32, 0f32, 0f32);
    }

    pub enum Direction {
        Front,
        Back,
        Up,
        Down,
        Left,
        Right,
    }

    const MOVEMENT_SPEED: f32 = 0.1;
    const MOUSE_SENSITIVITY: f32 = 0.005;
    struct RightHandCoordSys {
        front: Vec3,
    }

    impl RightHandCoordSys {
        const GLOBAL_UP: Vec3 = Directions::UP;

        pub fn new(front: Vec3) -> Self {
            Self { front }
        }

        pub fn direction(&self, direction: &Direction) -> Vec3 {
            use std::borrow::Borrow;

            let left = self.front.cross(&Self::GLOBAL_UP);
            let up = glm::rotate_vec3(&self.front, 90.0f32.to_radians(), left.borrow());
            match direction {
                Direction::Front => self.front,
                Direction::Back => -self.front,
                Direction::Up => up,
                Direction::Down => -up,
                Direction::Left => -left,
                Direction::Right => left,
            }
        }
    }

    // general camera
    #[derive(Debug, Clone)]
    pub struct CameraPerspectiveState {
        aspect_ratio: f32,
        fovy: f32,
        z_near: f32,
        z_far: f32,
    }

    impl Default for CameraPerspectiveState {
        fn default() -> Self {
            let mut viewport = [0; 4];
            gb::call! {
                #[panic]
                unsafe {
                    gb::gl::raw::GetIntegerv(gb::gl::raw::VIEWPORT, viewport.as_mut_ptr());
                }
            }
            let [.., width, height] = viewport;

            Self::new(
                Camera::DEFAULT_FOVY.to_radians(),
                width as f32 / height as f32,
                Camera::DEFAULT_Z_NEAR,
                Camera::DEFAULT_Z_FAR,
            )
        }
    }

    impl CameraPerspectiveState {
        pub fn new(aspect_ratio: f32, fovy: f32, z_near: f32, z_far: f32) -> Self {
            Self {
                aspect_ratio,
                fovy,
                z_near,
                z_far,
            }
        }
    }

    impl PerspectiveMatrixProvider for CameraPerspectiveState {
        fn perspective_matrix(&self) -> Mat4 {
            glm::perspective(self.aspect_ratio, self.fovy, self.z_near, self.z_far)
        }
    }

    #[derive(Debug, Clone)]
    pub struct CameraViewState {
        pub looking_direction: Vec3,
        pub position: Vec3,
    }

    impl Default for CameraViewState {
        fn default() -> Self {
            let looking_direction = Directions::BACK;
            let position = glm::vec3(0.0, 0.0, -1f32);
            Self {
                looking_direction,
                position,
            }
        }
    }

    impl CameraViewState {
        pub fn new(looking_direction: Vec3, position: Vec3) -> Self {
            Self {
                looking_direction,
                position,
            }
        }
    }

    impl ViewMatrixProvider for CameraViewState {
        fn view_matrix(&self) -> glm::Mat4 {
            let looking_point = self.position + self.looking_direction;
            glm::look_at(&self.position, &looking_point, &Directions::UP)
        }
    }

    pub struct Camera {
        pub view: CameraViewState,
        perspective: CameraPerspectiveState,
    }

    impl Camera {
        const DEFAULT_FOVY: f32 = 60.0;
        const DEFAULT_Z_NEAR: f32 = 0.1;
        const DEFAULT_Z_FAR: f32 = 150.0;

        const SENSITIVITY: f32 = 0.5;
        const SPEED: f32 = 0.05;

        pub fn angle() -> f32 {
            f32::to_radians(10f32)
        }

        pub fn rotate(&mut self, x_rot: f32, y_rot: f32) {
            // self.view.looking_direction = glm::rotate_y_vec3(&self.view.looking_direction, y_rot * Self::SENSITIVITY);
            // let coord_sys = RightHandCoordSys::new(self.view.looking_direction);
            // let right = coord_sys.direction(&Direction::Right);
            // self.view.looking_direction = glm::rotate_vec3(&self.view.looking_direction

            self.view.looking_direction =
                glm::rotate_y_vec3(&self.view.looking_direction, y_rot * Self::SENSITIVITY);
            let coord_sys = RightHandCoordSys::new(self.view.looking_direction);
            let right = coord_sys.direction(&Direction::Right);
            self.view.looking_direction = glm::rotate_vec3(
                &self.view.looking_direction,
                x_rot * Self::SENSITIVITY,
                &right,
            );
        }

        pub fn r#move(&mut self, direction: &Direction) {
            let local = RightHandCoordSys::new(self.view.looking_direction);
            self.view.position += local.direction(direction) * Self::SPEED;
        }

        pub fn view_matrix(&self) -> glm::Mat4 {
            self.view.view_matrix()
        }

        pub fn perspective_matrix(&self) -> glm::Mat4 {
            self.perspective.perspective_matrix()
        }

        pub fn new(perspective: CameraPerspectiveState, view: CameraViewState) -> Self {
            Self { perspective, view }
        }
    }

    impl Default for Camera {
        fn default() -> Self {
            let perspective = CameraPerspectiveState::default();
            let view = CameraViewState::default();
            Self::new(perspective, view)
        }
    }
    pub trait ViewMatrixProvider {
        fn view_matrix(&self) -> glm::Mat4;
    }

    pub trait PerspectiveMatrixProvider {
        fn perspective_matrix(&self) -> glm::Mat4;
    }

    pub trait CameraProvider: PerspectiveMatrixProvider + ViewMatrixProvider {
        fn view_projection_matrix(&self) -> glm::Mat4;
    }

    impl<C: PerspectiveMatrixProvider + ViewMatrixProvider> CameraProvider for C {
        fn view_projection_matrix(&self) -> glm::Mat4 {
            self.perspective_matrix() * self.view_matrix()
        }
    }

    pub trait KinematicCamera: CameraProvider + Rotatable + FixedMovable {}

    impl<K: CameraProvider + Rotatable + FixedMovable> KinematicCamera for K {}

    // todo: move to kinematics
    pub trait Rotatable {
        fn rotate(&mut self, x_angle: f32, y_angle: f32);
    }

    pub trait FixedMovable {
        fn is_in_bounds(&self) -> bool {
            true
        }

        fn fixed_move(&mut self, direction: &Direction);
    }

    pub trait Movable {
        fn r#move(&mut self, vector: &Vec3);
    }

    #[derive(Default)]
    pub struct FreeRoamingCamera {
        pub camera: Camera,
    }

    impl FreeRoamingCamera {
        pub fn get_position(&self) -> Vec3 {
            self.camera.view.position
        }

        pub fn set_position(&mut self, new: Vec3) {
            self.camera.view.position = new;
        }
    }

    impl From<Camera> for FreeRoamingCamera {
        fn from(camera: Camera) -> Self {
            Self { camera }
        }
    }

    impl FixedMovable for FreeRoamingCamera {
        fn is_in_bounds(&self) -> bool {
            true
        }

        fn fixed_move(&mut self, direction: &Direction) {
            let position = self.camera.view.position.clone();
            self.camera.r#move(direction);
            if !self.is_in_bounds() {
                self.camera.view.position = position;
            }
        }
    }

    impl Rotatable for FreeRoamingCamera {
        fn rotate(&mut self, x_angle: f32, y_angle: f32) {
            self.camera.rotate(x_angle, y_angle);
        }
    }

    impl ViewMatrixProvider for FreeRoamingCamera {
        fn view_matrix(&self) -> glm::Mat4 {
            self.camera.view_matrix()
        }
    }

    impl PerspectiveMatrixProvider for FreeRoamingCamera {
        fn perspective_matrix(&self) -> glm::Mat4 {
            self.camera.perspective_matrix()
        }
    }
}

pub struct Timer(time::Instant);

impl Timer {
    pub fn new() -> Self {
        Self(time::Instant::now())
    }

    pub fn get(&self) -> time::Duration {
        std::time::Instant::now() - self.0
    }

    pub fn update(&mut self) -> time::Duration {
        let start = self.0;
        let now = std::time::Instant::now();
        self.0 = now;
        now - start
    }
}

pub trait Sample: Sized {
    fn initialize(
        window: window::Window,
        surface: surface::Surface<surface::WindowSurface>,
        context: context::PossiblyCurrentContext,
    ) -> anyhow::Result<Ctx<Self>>;

    fn render(&mut self);

    fn process_key(&mut self, code: winit::keyboard::KeyCode, state: winit::event::ElementState);

    fn process_mouse(&mut self, delta: (f64, f64));

    fn name() -> String;

    fn usage(&self) -> String;

    fn config() -> config::Config {
        config::Config::default()
    }
}

pub trait InteractiveSample: Sample {
    const FREQUENCY: usize;
    type DCtx;

    fn update(&mut self, dctx: &Self::DCtx, dt: Duration);
}

pub struct Ctx<T> {
    pub window: window::Window,
    pub surface: surface::Surface<surface::WindowSurface>,
    pub context: context::PossiblyCurrentContext,
    pub inner: T,
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
    physical_device_timer: Timer,
    ctx: Option<Ctx<T>>,
}

impl<T: Sample> Default for App<T> {
    fn default() -> Self {
        Self {
            physical_device_timer: Timer::new(),
            ctx: None,
        }
    }
}

impl<T: Sample> App<T> {
    fn init(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.ctx.is_none() {
            event_loop.set_control_flow(event_loop::ControlFlow::Poll);

            let config = T::config();

            // let icon = hello_textures::logo::uwr();
            // let icon: &[u8] = unsafe { std::slice::from_raw_parts(icon.as_ptr() as *const _, icon.len() * 4) };

            // let icon = winit::window::Icon::from_rgba(Vec::from_iter(icon.into_iter().map(Clone::clone)), 256, 256).unwrap();

            // Winit window creation
            let window_attributes = winit::window::WindowAttributes::default()
                // .with_window_icon(Some(icon))
                .with_inner_size(winit::dpi::PhysicalSize::new(config.width, config.height))
                .with_title(T::name())
                .with_resizable(false);

            // Glutin gl context initialization
            let version = context::Version::new(4, 6);
            println!(
                "initializing OpenGL {}.{} core",
                version.major, version.minor
            );
            let template = glutin::config::ConfigTemplateBuilder::new();
            let display_builder =
                DisplayBuilder::new().with_window_attributes(Some(window_attributes));

            let config_selector = |configs: Box<
                dyn Iterator<Item = glutin::config::Config> + '_,
            >| {
                configs
                    .reduce(|accum, config| {
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

            let (mut window, config) = display_builder
                .build(event_loop, template, config_selector)
                .expect("can create display");

            let raw_window_handle = window
                .as_ref()
                .and_then(|window| window.window_handle().map(|handle| handle.as_raw()).ok());

            let window = window.take().unwrap();

            window.set_cursor_grab(CursorGrabMode::Confined).ok();
            window.set_cursor_visible(false);

            let display = config.display();

            let context_attributes =
                context::ContextAttributesBuilder::new().build(raw_window_handle);

            let not_current_gl_context = unsafe {
                display
                    .create_context(&config, &context_attributes)
                    .expect("failed to create context")
            };

            let attrs = window
                .build_surface_attributes(<_>::default())
                .expect("Failed to build surface attributes");
            let surface = unsafe {
                config
                    .display()
                    .create_window_surface(&config, &attrs)
                    .unwrap()
            };

            let gl_context = not_current_gl_context
                .make_current(&surface)
                .expect("can make surface current");

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
            println!("press escape key to exit");
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

    fn process_key(&mut self, key: keyboard::KeyCode, state: ElementState) {
        if key == KeyCode::Escape {
            std::process::exit(0)
        }
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
    fn suspended(&mut self, _: &ActiveEventLoop) {}

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _: event::DeviceId,
        event: event::DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseMotion { delta } => self.process_mouse_input(delta),
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
                    ctx.window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        repeat,
                        ..
                    },
                ..
            } => {
                println!("{repeat}");
                self.process_key(key);
            }
            WindowEvent::CloseRequested => {
                std::process::exit(0);
            }
            WindowEvent::RedrawRequested => {
                // print!("\r{} Hz", 1000000 / self.physical_device_timer.get().as_micros());
                self.render()
            }
            _ => (),
        }
    }

    fn about_to_wait(&mut self, _: &ActiveEventLoop) {
        self.physical_device_timer.update();
    }

    fn resumed(&mut self, event_loop: &event_loop::ActiveEventLoop) {
        if self.ctx.is_none() {
            self.init(event_loop);
        }
    }
}

pub fn run_sample<S: Sample>() -> anyhow::Result<()> {
    let mut app = App::<S>::default();
    let event_loop = EventLoop::new()?;
    Ok(event_loop.run_app(&mut app)?)
}
