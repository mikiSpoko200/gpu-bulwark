use crate::impl_target;
use crate::gl;

/// Marker for types that represent Shader targets.
pub(crate) unsafe trait Target: gl::target::Target { }

/// Target for Vertex Shader stage.
pub enum Vertex { }

pub mod tesselation {
    /// Target for Tesselation Control Shader stage.
    pub enum Control { }

    /// Target for Tesselation Evaluation Shader stage.
    pub enum Evaluation { }
}

/// Target for represents Geometry Shader stage.
pub enum Geometry { }

/// Target for Fragment Shader stage.
pub enum Fragment { }

/// Target for Compute Shader stage.
pub enum Compute { }

impl_target!{ Vertex as VERTEX_SHADER }
impl_target!{ tesselation::Control as TESS_CONTROL_SHADER }
impl_target!{ tesselation::Evaluation as TESS_EVALUATION_SHADER }
impl_target!{ Geometry as GEOMETRY_SHADER }
impl_target!{ Fragment as FRAGMENT_SHADER }
impl_target!{ Compute as COMPUTE_SHADER }
