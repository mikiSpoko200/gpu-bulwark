
pub mod marker {
    use crate::glsl;
    pub trait Uniform: glsl::Type { }

    macro_rules! impl_uniform {
        ($tt: ty) => {
            impl Uniform for $tt { }
        };
        (vec $type: ident) => {
            impl<const SIZE: usize> Uniform for glsl::$type<SIZE>
            where
                glsl::Const<SIZE>: glsl::types::VecSize,
            {}
        }
    }

    impl_uniform!(f32);
    impl_uniform!(f64);
    impl_uniform!(i32);
    impl_uniform!(u32);
    impl_uniform!(bool);

    impl_uniform!(vec Vec);
    impl_uniform!(vec IVec);
    impl_uniform!(vec UVec);
    impl_uniform!(vec DVec);
    impl_uniform!(vec BVec);

    impl<T, const N: usize> Uniform for glsl::Array<T, N>
    where
        T: glsl::Type,
    {
    }
}
