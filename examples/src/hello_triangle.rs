#![allow(unused)]

use crate::Ctx;

use winit::window;
use glutin::{context, surface};
use gb::gl::{self, VertexArray};

use gl::shader;

pub struct Sample {
    program: gl::Program<(), (), (), ()>,
    vao: gl::VertexArray<()>,
}

impl crate::Sample for Sample {
    fn initialize(window: winit::window::Window, surface: glutin::surface::Surface<glutin::surface::WindowSurface>, context: glutin::context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        // Read shader source code.
        let vs_source = std::fs::read_to_string("shaders/hello_triangle.vert")?;
        let fs_source = std::fs::read_to_string("shaders/hello_triangle.frag")?;

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);

        // Defining shaders.
        let vs = uncompiled_vs
            .compile()?
            .into_main();
        let fs = uncompiled_fs
            .compile()?
            .into_main();

        let vao = VertexArray::default();

        // Building type checked pipeline.
        let program = gl::Program::builder()
            .no_uniforms()
            .no_resources()
            .vertex_main(&vs)
            .fragment_main(&fs)
            .build()?;
        Ok(Ctx {
            window,
            surface,
            context,
            inner: Self {
                program,
                vao,
            },
        })
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
