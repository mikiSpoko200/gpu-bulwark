#![allow(unused)]

pub mod attributes;
pub(crate) mod buffer;
pub mod prelude;
pub mod program;
pub mod resource;
pub mod shader;
pub mod vertex_array;

use crate::{
    gl_call, glsl,
    object::{
        program::{attach::AttachShared, Program},
        vertex_array::VertexArray,
    },
    target::buffer::format,
};
use frunk::HList;
use program::attach::AttachMain;

use self::{program::parameters, resource::Bindable};

fn test() {
    use crate::target::shader::{Fragment, Vertex};
    use buffer::{Draw, Static};
    use shader;
    use shader::Shader;

    let vs_source = stringify! {
        #version 330 core
        layout (location = 0) in vec3 position;

        out vec4 vertexColor;

        void add_one(const vec3* src, vec* dest);
        void sub_one(const vec3* src, vec* dest);

        void main()
        {
            gl_Position = vec4(position, 1.0);
            vertexColor = vec4(0.5, 0.0, 0.0, 1.0);
        }
    };

    let common_source = stringify! {
        #version 330 core

        void add_one(const vec3* src, vec* dest) {
            *dest = *src + vec3(1.0f, 1.0f, 1.0f);
        }

        void sub_one(const vec3* src, vec* dest) {
            *dest = *src - vec3(1.0f, 1.0f, 1.0f);
        }
    };

    let fs_source = stringify! {
        #version 330 core
        out vec4 FragColor;

        in vec4 vertexColor;

        void main()
        {
            FragColor = vertexColor;
        }
    };

    let foo: usize = 1;
    let uncompiled_vs = Shader::<Vertex>::create();
    let uncompiled_fs = Shader::<Fragment>::create();
    let common = Shader::<Vertex>::create();

    uncompiled_vs.source(&[vs_source]);
    uncompiled_fs.source(&[fs_source]);
    common.source(&[common_source]);

    let vs = uncompiled_vs
        .compile()
        .expect("valid shader code provided")
        .into_main()
        .input::<glsl::types::Vec3>()
        .input::<glsl::types::Vec4>()
        .output::<glsl::types::Vec4>();
    let fs = uncompiled_fs
        .compile()
        .expect("valid shader code provided")
        .into_main()
        .input::<glsl::types::Vec4>()
        .output::<glsl::types::Vec4>();
    let common = common
        .compile()
        .expect("valid shader code provided")
        .into_shared();

    let vs_main = vs;

    let program = Program::builder(&vs_main)
        .vertex_shared(&common)
        .fragment_main(&fs)
        .build()
        .expect("linking successful");

    let mut positions = buffer::Buffer::create();
    positions.data::<(Static, Draw)>(&[[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]]);

    let mut colors = buffer::Buffer::create();
    colors.data::<(Static, Draw)>(&[
        [1.0, 0.0, 0.0, 1.0],
        [0.0, 1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0, 1.0],
    ]);

    let vao = VertexArray::create()
        .attach::<0, _>(&positions)
        .attach::<1, _>(&colors);

    draw_arrays(&vao, &program);
}

pub fn draw_arrays<AS, PSI, PSO>(vao: &vertex_array::VertexArray<AS>, program: &Program<PSI, PSO>)
where
    AS: attributes::Attributes,
    PSI: parameters::Parameters,
    PSO: parameters::Parameters,
    (AS, PSI): glsl::compatible::Compatible<AS, PSI>,
{
    vao.bind();
    program.bind();

    
    gl_call! {
        #[panic]
        unsafe {
            gl::DrawArrays(gl::TRIANGLES, 0, vao.len() as _);
        }
    }
    vao.unbind();
    program.unbind();
}
