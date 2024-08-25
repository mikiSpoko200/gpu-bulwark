#![allow(unused)]

use crate::Ctx;


use gb::gl::texture::{self, pixel, TextureUnit};
use winit::window;
use glutin::{context, surface};
use gb::{gl, glsl};


use gl::vertex_array::Attribute;
use gl::shader;
use gl::buffer::{Static, Draw};
use gl::{Program, Buffer, VertexArray};
use glsl::MatchingInputs as _;


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

type Uniforms = glsl::Uniforms! {
    layout(location = 0) mat4;
};

type Resources = glsl::Uniforms! {
    layout(binding = 0) sampler2D;
};

type Attributes = gb::HList! {
    Attribute<[f32; 2], 0>,
    Attribute<[f32; 2], 1>,
};


use texture::{target::D2, Immutable, image::{Format, format}};

pub struct Sample {
    program: Program<Inputs, FsOutputs, Uniforms, Resources>,
    vao: VertexArray<Attributes>,
    texture: texture::TextureUnit<D2, Immutable<D2>, Format<format::RGB, 8>, 0>
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
    
        let mut positions = Buffer::create();
        positions.data::<(Static, Draw)>(&[[-0.5, -0.5], [0.5, -0.5], [0.0, 0.5f32]]);
    
        let mut texture_coords = Buffer::create();
        texture_coords.data::<(Static, Draw)>(&[[0.0, 0.0], [1.0, 0.0], [0.5, 1.0f32]]);
            
        let mut texture = texture::Texture::create_with_storage_2d(256, 256);

        let mut pixels = Self::generate_256x256_texture();
        
        println!("{:?}", &pixels[..10]);

        texture.sub_image_2d::<pixel::channels::RGB, _>(0..256, 0..256, &mut pixels);


        println!("{:?}", &pixels[..10]);


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
        // TODO: Update texture
    }
    
    fn process_mouse(&mut self, (dx, dy): (f64, f64)) {
        // TODO: Update texture
    }
}
