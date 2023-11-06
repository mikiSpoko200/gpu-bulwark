use crate::glsl::types::{Const, VecSize};
use super::types;

pub unsafe trait Compatible { }

unsafe impl<T, P, const N: usize> Compatible for (types::Vec<P, N>, T)
    where
        T: AsRef<[P; N]>,
        Const<N>: VecSize,
{}


// macro_rules! compatible {
//     (vec, $T1: path, $T2: path) => {
//         unsafe impl<P, const N: usize> Compatible for ($T1, [P; N])
//             where
//                 crate::glsl::types::Const<N>: crate::glsl::types::VecSize,
//                 P: crate::types::Primitive,
//         { }
//     };
//     (mat, $%)
// }
