use super::{Target, target};
use crate::gl::{self, Integer};

#[hi::marker]
pub trait ForBuffer<T: Target> { }

hi::denmark! { ForBuffer<target::ElementArray> where u8, u16, u32 }

pub trait ExtForElementBuffer: gl::Type<Kind = Integer> {}

impl<T> ExtForElementBuffer for T where T: ForBuffer<target::ElementArray> + gl::Type<Kind = Integer> {}
