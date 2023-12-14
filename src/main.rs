#![allow(unused_unsafe)]

use gl;
use winit;
// use glutin::event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
// use glutin::event_loop::{EventLoop, ControlFlow};
// use glutin::window::WindowBuilder;
// use glutin::{Api, GlRequest};
// use glutin::dpi::{LogicalSize, PhysicalPosition, Size};

mod error;
mod glsl;
mod object;
mod prelude;
mod target;
mod types;

fn main() {
    // let event_loop = EventLoop::new();
    // let window = WindowBuilder::new()
    //     .with_title("3D labyrinth")
    //     .with_min_inner_size(LogicalSize { width: 800, height: 600 });

    // let gl_context = glutin::ContextBuilder::new()
    //     .with_gl(GlRequest::Specific(Api::OpenGl, GL_VERSION))
    //     .build_windowed(window, &event_loop)
    //     .expect("Cannot create windowed context");

    // let gl_context = unsafe {
    //     gl_context
    //         .make_current()
    //         .expect("Failed to make context current")
    // };

    // gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

    // gl_context.window().set_cursor_visible(false);
    // println!("Hello, world!");

    // event_loop.run(move |event, _, control_flow| {
    //     *control_flow = ControlFlow::Wait;
    //     // todo: for smoother movement and better frame rates process all inputs once per each frame.

    //     match event {
    //         Event::LoopDestroyed => (),
    //         Event::WindowEvent { event, .. } => {
    //             match event {
    //                 WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
    //                 _ => (),
    //             }
    //         },
    //         Event::DeviceEvent { event, .. } => {
    //             match event {
    //                 Event::RedrawRequested(_) => {
    //                     gl_context.swap_buffers().unwrap();
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    // });
}
