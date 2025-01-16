use super::{Target, target};

#[hi::marker]
pub trait ForBuffer<T: Target> { }

hi::denmark! { ForBuffer<target::ElementArray> where u8, u16, u32 }
