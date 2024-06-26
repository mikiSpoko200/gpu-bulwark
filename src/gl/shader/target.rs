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

impl_target!{ Vertex as VERTEX_SHADER }
impl_target!{ tesselation::Control as TESS_CONTROL_SHADER }
impl_target!{ tesselation::Evaluation as TESS_EVALUATION_SHADER }
impl_target!{ Geometry as GEOMETRY_SHADER }
impl_target!{ Fragment as FRAGMENT_SHADER }
impl_target!{ Compute as COMPUTE_SHADER }
