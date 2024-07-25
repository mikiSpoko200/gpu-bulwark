use crate::impl_target;
use crate::gl;

/// Marker for types that represent Shader targets.
pub trait Target: gl::target::Target { }

/// Target for Vertex Shader stage.
#[hi::mark(Target)]
pub enum Vertex { }

pub mod tesselation {
    use super::*;

    /// Target for Tesselation Control Shader stage.
    #[hi::mark(Target)]
    pub enum Control { }

    /// Target for Tesselation Evaluation Shader stage.
    #[hi::mark(Target)]
    pub enum Evaluation { }
}

pub type TessControl = tesselation::Control;
pub type TessEvaluation = tesselation::Evaluation;

/// Target for represents Geometry Shader stage.
#[hi::mark(Target)]
pub enum Geometry { }

/// Target for Fragment Shader stage.
#[hi::mark(Target)]
pub enum Fragment { }

/// Target for Compute Shader stage.
#[hi::mark(Target)]
pub enum Compute { }

impl_target!{ Vertex as VERTEX_SHADER }
impl_target!{ tesselation::Control as TESS_CONTROL_SHADER }
impl_target!{ tesselation::Evaluation as TESS_EVALUATION_SHADER }
impl_target!{ Geometry as GEOMETRY_SHADER }
impl_target!{ Fragment as FRAGMENT_SHADER }
impl_target!{ Compute as COMPUTE_SHADER }
