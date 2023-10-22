//! TODO: Add DebugOnly type
//!
//!

#[derive(Debug, Clone, Copy, Hash, Default, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub(crate) struct u31 {
    inner: i32
}

impl u31 {
    pub const fn new(inner: i32) -> Self {
        if inner < 0 {
            panic!("value must be non negative")
        }
        Self { inner }
    }

    pub const fn get(self) -> i32 {
        self.inner
    }
}
