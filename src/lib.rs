
pub mod constraint;
pub mod ext;
pub mod glsl;
pub mod hlist;
pub mod gl;
pub mod prelude;
pub mod ffi;
pub mod utils;
pub mod md;
pub mod ts;
pub mod valid;

pub fn load_with(loader: impl FnMut(&'static str) -> *const std::os::raw::c_void) {
    glb::load_with(loader);
}
