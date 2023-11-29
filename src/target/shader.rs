use crate::impl_target;

/// Marker for types that represent Shader targets.
pub(crate) unsafe trait Target: crate::target::Target {}

/// Zero-sized struct that represents Vertex Shader stage.
#[derive(Default)]
pub struct Vertex;

pub mod tesselation {
    /// Zero-sized struct that represents Tesselation Control Shader stage.
    #[derive(Default)]
    pub struct Control;

    /// Zero-sized struct that represents Tesselation Evaluation Shader stage.
    #[derive(Default)]
    pub struct Evaluation;
}

/// Zero-sized struct that represents Geometry Shader stage.
#[derive(Default)]
pub struct Geometry;

/// Zero-sized struct that represents Fragment Shader stage.
#[derive(Default)]
pub struct Fragment;

/// Zero-sized struct that represents Compute Shader stage.
#[derive(Default)]
pub struct Compute;

impl_target!(shader, Vertex, VERTEX_SHADER);
impl_target!(shader, tesselation::Control, TESS_CONTROL_SHADER);
impl_target!(shader, tesselation::Evaluation, TESS_EVALUATION_SHADER);
impl_target!(shader, Geometry, GEOMETRY_SHADER);
impl_target!(shader, Fragment, FRAGMENT_SHADER);
impl_target!(shader, Compute, COMPUTE_SHADER);
