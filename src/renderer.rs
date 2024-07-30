use winit::application;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::DeviceIdExtWindows;
use winit::window::{Window, WindowId};

use crate::glsl;

use crate::gl::{buffer, buffer::Buffer};

type Inputs = crate::Bindings! {
    layout(location = 0) in vec3;
    layout(location = 1) in vec4;
    layout(location = 2) in vec2;
};

type Outputs = crate::Outputs! {
    layout(location = 0) vec4;
};

type Uniforms = crate::Uniforms! {
    layout(location = 0) mat4;
};

pub struct Buffers {
    positions: Buffer<buffer::Array, glsl::Vec4>,
}

pub struct Renderer {
    window: Option<Window>,
    program: crate::gl::program::Program<Inputs, Outputs, Uniforms>,
    // buffers:
}

#[allow(unused)]
impl application::ApplicationHandler for Renderer {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        todo!()
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                println!(
                    "device id: {}",
                    device_id
                        .persistent_identifier()
                        .expect("device has a persistent identifier")
                );
            }
            _ => {}
        }
    }
}
