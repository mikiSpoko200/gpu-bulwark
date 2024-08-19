/// 

use crate::prelude::internal::*;

/// Provide information about layout in the buffer?
/// This will be implemented by for example `Interleaved`, `Subdivided`
pub trait Layout {
    const STRIDE: usize;
}
