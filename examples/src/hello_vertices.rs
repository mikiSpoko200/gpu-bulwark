
#![allow(unused)]

use crate::Ctx;

pub struct Sample { }

impl crate::Sample for Sample {
    fn initialize(window: winit::window::Window, surface: glutin::surface::Surface<glutin::surface::WindowSurface>, context: glutin::context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        todo!()
    }

    fn render(&mut self) {
        todo!()
    }

    fn process_key(&mut self, code: winit::keyboard::KeyCode) {
        todo!()
    }

    fn process_mouse(&mut self, delta: (f64, f64)) {
        todo!()
    }
}
