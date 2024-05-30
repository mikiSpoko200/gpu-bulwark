use winit::application;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::windows::DeviceIdExtWindows;
use winit::window::{Window, WindowId};

use crate::glsl;

use crate::object::{buffer, buffer::Buffer, program::Program, shader, shader::Shader};

type Inputs = crate::Inputs! {
    layout(location = 0) glsl::Vec3;
    layout(location = 1) glsl::Vec4;
    layout(location = 2) glsl::Vec2;
};

type Outputs = crate::Outputs! {
    layout(location = 0) glsl::Vec4;
};

type Uniforms = crate::Uniforms! {
    layout(location = 0) glsl::Mat4;
};

pub struct Buffers {
    positions: Buffer<buffer::target::Array, glsl::Vec4>,
}

pub struct Renderer {
    window: Option<Window>,
    program: crate::object::program::Program<Inputs, Outputs, Uniforms>,
    // buffers:
}

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
