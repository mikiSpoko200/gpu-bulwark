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
use texture::pixel::channels::Channels as _;
use glsl::MatchingInputs as _;

// imports for reading images
use std::io::Cursor;
use slice_of_array::prelude::*;

pub mod logo {
    use std::path::Path;

    use image::{DynamicImage, GenericImage, Pixel, RgbImage, Rgba};

    pub fn try_load_bitmap(path: &str) -> image::ImageResult<RgbImage> {
        image::open(path).map(|dynamic| dynamic.into_rgb8())
    }

    pub fn load_bitmap_from_resources(path: &str) -> RgbImage {
        try_load_bitmap(path).expect("resource paths are valid")
    }

    pub fn uwr(dest: &mut RgbImage) {
        dest.copy_from(&load_bitmap_from_resources("resources/uwr.bmp"), 0, 0).expect("image storage matches loaded bitmaps")
    }

    pub fn rust(dest: &mut RgbImage) {
        dest.copy_from(&load_bitmap_from_resources("resources/rust.bmp"), 0, 0).expect("image storage matches loaded bitmaps")
    }

    pub fn opengl(dest: &mut RgbImage) {
        dest.copy_from(&load_bitmap_from_resources("resources/opengl.bmp"), 0, 0).expect("image storage matches loaded bitmaps")
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

type Resources = glsl::Glsl! {
    layout(binding = 0) uniform sampler2D;
};

type Attributes = gb::HList! {
    Attribute<[f32; 2], 0>,
    Attribute<[f32; 2], 1>,
};


use texture::{target::D2, Immutable, image::{Format, format}};

pub struct Sample {
    program: Program<Inputs, FsOutputs, (), Resources>,
    vao: VertexArray<Attributes>,
    texture: texture::TextureUnit<D2, Immutable<D2>, Format<format::RGB, u8>, 0>,
    image: image::RgbImage,
}

impl Sample {
    const TEXTURE_SIZE: usize = 256;
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

        let glsl::vars![ sampler ] = Resources::default();

        let mut uncompiled_vs = shader::create::<shader::target::Vertex>();
        let mut uncompiled_fs = shader::create::<shader::target::Fragment>();
    
        uncompiled_vs.source(&[&vs_source]);
        uncompiled_fs.source(&[&fs_source]);
    
        let vs = uncompiled_vs
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
        .no_uniforms()
        .resources(|resources| resources
            .sampler(&sampler)
        )
        .vertex_main(&vs)
        .fragment_main(&fs)
        .build()?;

        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(
            &[[0.5, -0.5], [ 0.5, 0.5], [-0.5, -0.5], [-0.5, 0.5], [0.5, 0.5], [-0.5, -0.5]]
        );
        
        let mut texture_coords = Buffer::create();
        texture_coords.data::<(Static, Draw)>(
            &[[1.0, 1.0], [1.0, 0.0], [0.0, 1.0], [0.0, 0.0], [1.0, 0.0], [0.0, 1.0]]
        );

        let mut image = image::RgbImage::new(Self::TEXTURE_SIZE as _, Self::TEXTURE_SIZE as _);
        let mut texture = texture::Texture::create_with_storage_2d(Self::TEXTURE_SIZE, Self::TEXTURE_SIZE);

        logo::uwr(&mut image);
        
        texture.sub_image_2d::<pixel::channels::RGB, _>(0..Self::TEXTURE_SIZE, 0..Self::TEXTURE_SIZE, &image.nest::<[_; 3]>());

        let vao = VertexArray::create()
            .vertex_attrib_pointer(&vin_position, positions)
            .vertex_attrib_pointer(&vin_tex, texture_coords);
        
        let inner = Self {
            program,
            vao,
            texture: TextureUnit::<_, _, _, 0>::new(texture).expect("texture unit 0 is bindable"),
            image,
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

        let mut update_image = |load: fn(&mut image::RgbImage), message: &str| {
            print!("\rdisplaying logo {:>6}", message);
            std::io::stdout().flush().unwrap();
            load(&mut self.image);
            texture.sub_image_2d::<pixel::channels::RGB, _>(0..Self::TEXTURE_SIZE, 0..Self::TEXTURE_SIZE, &self.image.nest::<[_; 3]>());
        };

        match code {
            winit::keyboard::KeyCode::KeyA => update_image(logo::uwr, "uwr"),
            winit::keyboard::KeyCode::KeyS => update_image(logo::opengl, "opengl"),
            winit::keyboard::KeyCode::KeyD => update_image(logo::rust, "rust"),
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
