#![allow(unused)]

use std::io::Write;
use std::marker::PhantomData;

use crate::{common, Ctx};


use gb::gl::texture::{self, pixel, TextureUnit};
use winit::window;
use glutin::{context, surface};
use gb::{gl, glsl};


use gl::vertex_array::Attribute;
use gl::shader;
use gl::buffer::{Static, Draw};
use gl::{Program, Buffer, VertexArray};
use glsl::MatchingInputs as _;


pub mod logo {
    const BITMAP_OFFSET: usize = 54;
    const RAW_BITMAP_SIZE: usize = 262198;

    type Bitmap = [u8; RAW_BITMAP_SIZE];

    static UWR: &'static Bitmap = include_bytes!("../resources/uwr.bmp");
    static RUST: &'static Bitmap = include_bytes!("../resources/rust.bmp");
    static OPENGL: &'static Bitmap = include_bytes!("../resources/opengl.bmp");

    pub fn pixels(bmp: &'static Bitmap) -> &'static [[u8; 4]] {
        /// SAFETY: data layout is correct
        unsafe { std::mem::transmute(&bmp[BITMAP_OFFSET..]) }
    }

    pub fn uwr() -> &'static [[u8; 4]] {
        pixels(&UWR)
    }

    pub fn rust() -> &'static [[u8; 4]] {
        pixels(RUST)
    }

    pub fn opengl() -> &'static [[u8; 4]] {
        pixels(OPENGL)
    }
}

type Inputs = glsl::Inputs! {
    layout(location = 0) vec2;
    layout(location = 1) vec2;
};

type VsOutputs = glsl::Outputs! {
    layout(location = 0) vec2;
};

type FsOutputs = glsl::Outputs! {
    layout(location = 0) vec4;
};

type Uniforms = glsl::Glsl! {
    layout(location = 0) uniform mat4;
};

type Resources = glsl::Glsl! {
    layout(binding = 0) uniform sampler2D;
};

type Attributes = gb::HList! {
    Attribute<[f32; 2], 0>,
    Attribute<[f32; 2], 1>,
};


use texture::{target::D2, Immutable, image::{Format, format}};

pub struct Sample {
    program: Program<Inputs, FsOutputs, Uniforms, Resources>,
    vao: VertexArray<Attributes>,
    texture: texture::TextureUnit<D2, Immutable<D2>, Format<format::RGBA, u8>, 0>
}

impl Sample {
    fn generate_256x256_texture() -> Vec<[u8; 3]> {
        let width = 256;
        let height = 256;
        let mut texture = Vec::with_capacity(width * height);
    
        for y in 0..height {
            for x in 0..width {
                let pixel = [
                    (x as u8).wrapping_add(y as u8),
                    y as u8,
                    x as u8,
                ];
                texture.push(pixel);
            }
        }
        texture
    }
}

impl crate::Sample for Sample {
    fn initialize(window: window::Window, surface: surface::Surface<surface::WindowSurface>, context: context::PossiblyCurrentContext) -> anyhow::Result<Ctx<Self>> {
        let vs_source = std::fs::read_to_string("shaders/hello_textures.vert")?;
        let fs_source = std::fs::read_to_string("shaders/hello_textures.frag")?;

        let vs_inputs = Inputs::default();
        let glsl::vars![vin_position, vin_tex] = &vs_inputs;

        let vs_outputs = VsOutputs::default();

        let fs_inputs = vs_outputs.matching_inputs();
        let glsl::vars![ fs_output ] = FsOutputs::default();

        let glsl::vars![ view_matrix_location ] = Uniforms::default();
        let glsl::vars![ sampler ] = Resources::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);
    
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
            .output(&fs_output);

        let program = Program::builder()
        .uniforms(|definitions| definitions
            .define(&view_matrix_location, &[[0f32; 4]; 4])
        )
        .resources(|resources| resources
            .sampler(&sampler)
        )
        .vertex_main(&vs)
        .uniforms(|matcher| matcher.bind(&view_matrix_location))
        .fragment_main(&fs)
        .build()?;

        // let mut positions = Buffer::create();
        // positions.data::<(Static, Draw)>(
        //     &[[0.5, -0.5], [ 0.5, 0.5], [-0.5, -0.5], [-0.5, 0.5], [0.5, 0.5], [-0.5, -0.5]]
        // );
        
        // let mut texture_coords = Buffer::create();
        // texture_coords.data::<(Static, Draw)>(
        //     &[[ 1.0, 0.0], [ 1.0, 1.0], [ 0.0, 0.0], [ 0.0, 1.0], [ 1.0, 1.0], [ 0.0, 0.0]]
        // );  
        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(
            &[[0.5, -0.5], [ 0.5, 0.5], [-0.5, -0.5], [-0.5, 0.5], [0.5, 0.5], [-0.5, -0.5]]
        );
        
        let mut texture_coords = Buffer::create();
        texture_coords.data::<(Static, Draw)>(
            &[[ 1.0, 0.0], [ 1.0, 1.0], [ 0.0, 0.0], [ 0.0, 1.0], [ 1.0, 1.0], [ 0.0, 0.0]]
        );  

        let mut texture = texture::Texture::create_with_storage_2d(256, 256);

        let pixels = logo::uwr();
        
        texture.sub_image_2d::<pixel::channels::BGRA, _>(0..256, 0..256, &pixels);


        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_tex, texture_coords);
        
        let inner = Self {
            program,
            vao,
            texture: TextureUnit::<_, _, _, 0>::new(texture).expect("texture unit 0 is bindable")
        };

        Ok(Ctx {
            window,
            surface,
            context,
            inner,
        })
    }
    
    fn render(&mut self) {
        let texture_bindings = texture::TextureUnits::default()
            .add(&self.texture);
        
        gl::call! {
            [panic]
            unsafe {
                gl::raw::ClearColor(0.4, 0.5, 0.6, 1.0);
                gl::raw::Clear(gl::raw::COLOR_BUFFER_BIT);
            }
        }
        self.program.draw_arrays_ext(&self.vao, &texture_bindings);
        
    }
    
    fn process_key(&mut self, code: winit::keyboard::KeyCode) {
        let texture = &mut self.texture;

        let mut update_image = |pixels: &[[u8; 4]], message: &str| {
            print!("\rdisplaying logo {:>6}", message);
            std::io::stdout().flush().unwrap();
            texture.sub_image_2d::<pixel::channels::BGRA, _>(0..256, 0..256, &pixels);
        };

        match code {
            winit::keyboard::KeyCode::KeyA => update_image(logo::uwr(), "uwr"),
            winit::keyboard::KeyCode::KeyS => update_image(logo::opengl(), "opengl"),
            winit::keyboard::KeyCode::KeyD => update_image(logo::rust(), "rust"),
            _ => (),
        }
    }
    
    fn process_mouse(&mut self, _: (f64, f64)) { }
    
    fn config() -> common::config::Config {
        common::config::Config {
            width: 512,
            height: 512,
        }
    }
    
    fn usage(&self) -> String {
        String::from("use A, S, D keys to change displayed texture")
    }
    
    fn name() -> String {
        String::from("hello-textures")
    }
}
