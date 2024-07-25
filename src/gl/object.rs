
use crate::prelude::internal::*;
use crate::error;
use crate::gl;

pub struct Bind<B: Binder>(PhantomData<B>);

impl<B: Binder> Bind<B> {
    pub(super) fn new(name: u32) -> Self {
        B::bind(name);
        Self(PhantomData)
    }
}

impl<B: Binder> Drop for Bind<B> {
    fn drop(&mut self) {
        B::unbind();
    }
}

pub(crate) trait Binder: Sized {
    fn bind(name: u32);
    fn unbind() {
        Self::bind(0);
    }
}

// macro_rules! impl_multiple_binders {
//     ($($type_vars:ident),+ ; $($vars:ident),+) => {
//         impl<$($type_vars),+> Bind for ($($type_vars),+)
//         where
//             $($type_vars: Bind),+
//         {
//             fn bind(&self) {
//                 let ($($vars),+) = self;
//                 $($vars.bind());+ ;
//             }

//             fn unbind(&self) {
//                 let ($($vars),+) = self;
//                 $($vars.unbind());+ ;
//             }
//         }
//     };
// }

// impl_multiple_binders! { T1, T2                              ; t1, t2                             }
// impl_multiple_binders! { T1, T2, T3                          ; t1, t2, t3                         }
// impl_multiple_binders! { T1, T2, T3, T4                      ; t1, t2, t3, t4                     }
// impl_multiple_binders! { T1, T2, T3, T4, T5                  ; t1, t2, t3, t4, t5                 }
// impl_multiple_binders! { T1, T2, T3, T4, T5, T6              ; t1, t2, t3, t4, t5, t6             }
// impl_multiple_binders! { T1, T2, T3, T4, T5, T6, T7          ; t1, t2, t3, t4, t5, t6, t7         }
// impl_multiple_binders! { T1, T2, T3, T4, T5, T6, T7, T8      ; t1, t2, t3, t4, t5, t6, t7, t8     }
// impl_multiple_binders! { T1, T2, T3, T4, T5, T6, T7, T8, T9  ; t1, t2, t3, t4, t5, t6, t7, t8, t9 }

pub unsafe trait Allocator: Sized {
    fn allocate(names: &mut [u32]);

    fn free(names: &[u32]);
}

pub(crate) trait Object: Sized + std::ops::Deref<Target = ObjectBase<Self>> {
    type Binder: Binder;
    type Allocator: Allocator;
}

#[repr(transparent)]
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct ObjectBase<O: Object> {
    name: u32,
    object: PhantomData<O>,
}

impl<O: Object> Default for ObjectBase<O> {
    fn default() -> Self {
        let mut name = 0;
        O::Allocator::allocate(std::slice::from_mut(&mut name));
        Self {
            name,
            object: PhantomData,    
        }
    }
}

impl<O: Object> Drop for ObjectBase<O> {
    fn drop(&mut self) {
        O::Allocator::free(&[self.name]);
    }
}

impl<O: Object> ObjectBase<O> {
    pub fn name(&self) -> u32 {
        self.name
    }

    pub fn bind(&self) -> Bind<O::Binder> {
        Bind::new(self.name())
    }
}
