use std::mem;
use gl::types::GLuint;

pub struct Handle<R: Resource> {
    resource: R
}

impl<R> Drop for Handle<R> where R: Resource {
    fn drop(&mut self) {
        todo!("Drop takes &mut not owned value :/");
        self.resource.delete()
    }
}

pub trait Resource: Sized {
    fn const_bulk_create<const N: usize>() -> [Self; N];

    fn dyn_bulk_create(n: usize) -> Vec<Self>;

    /// Delete Self in bulk by passing an arbitrary contiguous array of Self.
    ///
    /// note: the 'static bound should ensure that said array is moved.
    fn bulk_delete<R>(resources: R) where R: AsRef<[Self]> + 'static;

    fn delete(self) /* note: Figure out how to remove this bound */ where Self: 'static {
        Self::bulk_delete([self]);
    }
}

#[repr(transparent)]
struct Buffer {
    name: GLuint,
}

impl Buffer {
    pub fn new(name: GLuint) -> Self {
        Self { name }
    }
}

impl Resource for Buffer {
    fn const_bulk_create<const N: usize>() -> [Self; N] {
        let mut names: [GLuint; N] = [0; N];

        unsafe {
            gl::GenBuffers(N as _, &mut names as *mut _);
        }
        names.map(Self::new)
    }

    fn dyn_bulk_create(n: usize) -> Vec<Self> {
        let mut names = Vec::with_capacity(n);
        unsafe {
            gl::GenBuffers(n as _, names.as_mut_ptr());
        }
        unsafe { mem::transmute::<Vec<_>, Vec<Self>>(names) }
    }

    fn bulk_delete<R>(resources: R) where R: AsRef<[Self]> + 'static {
        // SAFETY: slice is ABI compatible with contiguous array and length is valid
        unsafe {
            gl::DeleteBuffers(resources.as_ref().len() as _,
            // SAFETY: Buffer is `#repr(transparent)` with GLuint
            unsafe {
                mem::transmute::<*const Self, *const GLuint>(resources.as_ref().as_ptr())
                }
            )
        }
    }
}
