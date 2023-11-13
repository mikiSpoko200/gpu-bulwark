#![allow(unused)]

use crate::object::resource::Handle;
use crate::{targets as target};

pub(crate) mod buffer;
pub mod prelude;
pub mod program;
pub mod resource;
pub mod shader;
pub mod vertex_array;

use target::buffer::format;
use crate::object::shader::Vertex;

fn test() {
    use buffer::{Draw, Static};
    use shader::{Vertex, Fragment};
    use shader;

    let vs_source = stringify! {
        #version 330 core
        layout (location = 0) in vec3 aPos;

        out vec4 vertexColor;

        void main()
        {
            gl_Position = vec4(aPos, 1.0);
            vertexColor = vec4(0.5, 0.0, 0.0, 1.0);
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

    let vs = shader::make::<Vertex>()
        .source(&[vs_source])
        .compile()
        .expect("valid shader code provided");
    let fs = shader::make::<Fragment>()
        .source(&[fs_source])
        .compile()
        .expect("valid shader code provided");

    let mut buffer = buffer::make();
    buffer.data::<(Static, Draw)>(&[
        [-0.5, -0.5, 0.0],
        [ 0.5, -0.5, 0.0],
        [ 0.0,  0.5, 0.0],
    ]);




}
