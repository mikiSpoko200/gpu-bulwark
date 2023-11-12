#![allow(unused)]

use crate::object::resource::Handle;
use crate::targets as target;

mod buffer;
mod prelude;
mod resource;
mod shader;
pub mod program;
mod vertex_array;


use target::buffer::format;

fn make<Data>() -> Handle<buffer::Buffer<target::buffer::Array, Data>>
    where
        (target::buffer::Array, Data): format::Valid
{
    Handle::new()
}

fn test() {
    use buffer::{Static, Draw};

    let buffer = make();
    buffer.data::<(Static, Draw)>(&[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]);
}