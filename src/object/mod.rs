#![allow(unused)]

use crate::object::resource::Handle;
use crate::targets as target;

mod buffer;
mod prelude;
pub mod program;
mod resource;
mod shader;
mod vertex_array;

use target::buffer::format;

fn make<Data>() -> Handle<buffer::Buffer<target::buffer::Array, Data>>
where
    (target::buffer::Array, Data): format::Valid,
{
    Handle::new()
}

fn test() {
    use buffer::{Draw, Static};

    let buffer = make();
    buffer.data::<(Static, Draw)>(&[[1.0, 1.0, 1.0], [1.0, 1.0, 1.0], [1.0, 1.0, 1.0]]);
}
