use crate::object::attributes::{AttributeDecl, Attribute};
use crate::object::buffer::target;

use super::prelude::InParameterBinding;
use crate::glsl;

pub use marker::Compatible;
pub use marker::hlist;

mod marker {
    use crate::glsl;

    pub trait Compatible<T>: Clone + Sized + glsl::FFI
    where
        T: glsl::FFI<Primitive=Self::Primitive>,
    {
        const CHECK_SAME_SIZE: ();
        fn as_pod(&self) -> &[T::Primitive];
    }
    
    pub mod hlist {
        pub trait Compatible<T> { }
    }
}

macro_rules! compatible {
    (scalar $gl: ty => $glsl: path) => {
        impl marker::Compatible<$glsl> for $gl {
            const CHECK_SAME_SIZE: () = assert!({<$gl as glsl::FFI>::SIZE} == {<$glsl as glsl::FFI>::SIZE});
            fn as_pod(&self) -> &[<$glsl as glsl::FFI>::Primitive] {
                std::slice::from_ref(self)
            }
        }
    };
    ($gl: ty => $glsl: path) => {
        impl marker::Compatible<$glsl> for $gl {
            const CHECK_SAME_SIZE: () = assert!(<$gl as glsl::FFI>::SIZE == <$glsl as glsl::FFI>::SIZE);
            fn as_pod(&self) -> &[<$glsl as glsl::FFI>::Primitive] {
                self
            }
        }
    };
}

// --------==========[ Rust base types ]==========--------

/// This is causing more trouble then its wodth
unsafe impl<S, const N: usize> glsl::FFI for [S; N]
where
    S: glsl::FFI,
{
    type Primitive = S::Primitive;

    const SIZE: usize = N;
}

compatible!{ scalar f32 => f32 }
compatible!{ scalar f64 => f64 }
compatible!{ scalar i32 => i32 }
compatible!{ scalar u32 => u32 }


compatible!{ [f32; 2] => glsl::Vec2 }
compatible!{ [f32; 3] => glsl::Vec3 }
compatible!{ [f32; 4] => glsl::Vec4 }

compatible!{ [i32; 2] => glsl::IVec2 }
compatible!{ [i32; 3] => glsl::IVec3 }
compatible!{ [i32; 4] => glsl::IVec4 }

compatible!{ [u32; 2] => glsl::UVec2 }
compatible!{ [u32; 3] => glsl::UVec3 }
compatible!{ [u32; 4] => glsl::UVec4 }

compatible!{ [f64; 2] => glsl::DVec2 }
compatible!{ [f64; 3] => glsl::DVec3 }
compatible!{ [f64; 4] => glsl::DVec4 }

// OpenGL bools are wierd (32 bit not `_Bool`)
// compatible!([i32; 2], glsl::BVec2);
// compatible!([i32; 3], glsl::BVec3);
// compatible!([i32; 4], glsl::BVec4);
// compatible!([u32; 2], glsl::BVec2);
// compatible!([u32; 3], glsl::BVec3);
// compatible!([u32; 4], glsl::BVec4);

compatible!{ [f32; 4] => glsl::Mat2x2 }
compatible!{ [f32; 6] => glsl::Mat2x3 }
compatible!{ [f32; 8] => glsl::Mat2x4 }

compatible!{ [f32; 6]  => glsl::Mat3x2 }
compatible!{ [f32; 9]  => glsl::Mat3x3 }
compatible!{ [f32; 12] => glsl::Mat3x4 }

compatible!{ [f32; 8]  => glsl::Mat4x2 }
compatible!{ [f32; 12] => glsl::Mat4x3 }
compatible!{ [f32; 16] => glsl::Mat4x4 }

compatible!{ [f64; 4] => glsl::DMat2x2 }
compatible!{ [f64; 6] => glsl::DMat2x3 }
compatible!{ [f64; 8] => glsl::DMat2x4 }

compatible!{ [f64; 6]  => glsl::DMat3x2 }
compatible!{ [f64; 9]  => glsl::DMat3x3 }
compatible!{ [f64; 12] => glsl::DMat3x4 }

compatible!{ [f64; 8]  => glsl::DMat4x2 }
compatible!{ [f64; 12] => glsl::DMat4x3 }
compatible!{ [f64; 16] => glsl::DMat4x4 }


/// This conflicts with blanket `glsl::FFI` used by `compatible!` above
// impl<T, const ROW: usize, const COL: usize> Compatible<glsl::Mat<T, ROW, COL>> for [[T; COL]; ROW]
// where
//     T: Primitive
// {
//     const CHECK_SAME_SIZE: () = assert!(<Self as glsl::FFI>::SIZE == <glsl::Mat<T, ROW, COL> as glsl::FFI>::SIZE);

//     fn as_pod(&self) -> &[<glsl::Mat<T, ROW, COL> as glsl::FFI>::Primitive] {
//         todo!()
//     }
// }

// --------==========[ nalgebra types ]==========--------

#[cfg(feature = "nalgebra")]
mod nalgebra {

}

#[cfg(feature = "nalgebra-glm")]
mod nalgebra_glm {
    use nalgebra_glm as glm;
    use crate::glsl;
    
    unsafe impl<T, const ROW: usize, const COL: usize> glsl::FFI for glm::TMat<T, ROW, COL>
    where
        T: glsl::marker::ScalarType,
    {
        type Primitive = T;
        const SIZE: usize = ROW * COL;
    }

    impl<T, const SIZE: usize> super::Compatible<glsl::base::Vec<T, SIZE>> for glm::TVec<T, SIZE>
    where
        T: glsl::marker::ScalarType,
        glsl::Const<SIZE>: glsl::VecSize,
        glsl::base::Vec<T, SIZE>: glsl::location::marker::Location,
        Self: AsRef<[T; SIZE]>,
    {
        const CHECK_SAME_SIZE: () = assert!(<Self as glsl::FFI>::SIZE == <glsl::base::Vec<T, SIZE> as glsl::FFI>::SIZE);
    
        fn as_pod(&self) -> &[T] {
            self.as_ref()
        }
    }

    impl<T, const ROW: usize, const COL: usize> super::Compatible<glsl::Mat<T, ROW, COL>> for glm::TMat<T, ROW, COL>
    where
        T: glsl::marker::ScalarType,
        glsl::Const<ROW>: glsl::VecSize,
        glsl::Const<COL>: glsl::VecSize,
        glsl::Mat<T, ROW, COL>: glsl::Type<Primitive = T>,
        Self: AsRef<[[T; COL]; ROW]>,
    {
        const CHECK_SAME_SIZE: () = assert!(<Self as glsl::FFI>::SIZE == <glsl::Mat<T, ROW, COL> as glsl::FFI>::SIZE);
    
        fn as_pod(&self) -> &[T] {
            unsafe { std::slice::from_raw_parts(self.as_ref().as_ptr() as *const _, ROW * COL)}
        }
    }
}



// --------==========[ Arrays of Compatible types ]==========--------


unsafe impl<S> glsl::FFI for &S
where
    S: glsl::FFI,
{
    type Primitive = S::Primitive;

    const SIZE: usize = 1;
}

impl<GL, GLSL, const N: usize> marker::Compatible<glsl::Array<GLSL, N>> for &GL
where
    GL: Compatible<GLSL>,
    GLSL: super::Type<Primitive=GL::Primitive>,
{
    const CHECK_SAME_SIZE: () = assert!(<Self as glsl::FFI>::SIZE == <glsl::Array<GLSL, N> as glsl::FFI>::SIZE);
    fn as_pod(&self) -> &[GLSL::Primitive] {
        unsafe { std::slice::from_raw_parts(<Self as marker::Compatible<glsl::glsl::Array<GLSL, N>>>::as_pod(self).as_ptr() as *const _, GLSL::SIZE) }
    }
}

/// NOTE: This won't work sin
impl<GLSL, GL, const N: usize> marker::Compatible<glsl::Array<GLSL, N>> for [GL; N]
where
    GL: Compatible<GLSL>,
    GLSL: super::Type<Primitive = GL::Primitive>,
{
    const CHECK_SAME_SIZE: () = assert!(<Self as glsl::FFI>::SIZE == <glsl::Array<GLSL, N> as glsl::FFI>::SIZE);
    fn as_pod(&self) -> &[GLSL::Primitive] {
        unsafe { std::slice::from_raw_parts(<Self as marker::Compatible<glsl::glsl::Array<GLSL, N>>>::as_pod(self).as_ptr() as *const _, GLSL::SIZE) }
    }
}


// --------==========[ HList integration ]==========--------


impl marker::hlist::Compatible<()> for () { }


impl<'buffers, AS, A, PS, P, const ATTRIBUTE_INDEX: usize> marker::hlist::Compatible<(PS, InParameterBinding<P, ATTRIBUTE_INDEX>)> for (AS, AttributeDecl<'buffers, A, ATTRIBUTE_INDEX>)
where
    A: Attribute + glsl::FFI<Primitive = <P as glsl::FFI>::Primitive>,
    A: marker::Compatible<P> + target::format::Valid<target::Array>,
    AS: marker::hlist::Compatible<PS>,
    P: glsl::Type,
{ }
