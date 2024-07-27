use std::fmt::{Display, Formatter};

use glam::Vec2;

use crate::renderer::coordinates::WGSLRepr;

///Describes a point on a texture, canvas or square area relative to its size, where a value of 0.0
/// means top/left corner and 1.0 means bottom/right
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct PTexCoords {
  pub x: f32,
  pub y: f32,
}

impl PTexCoords {
  pub fn new(x: f32, y: f32) -> Self {
    Self { x, y }
  }
}

impl Display for PTexCoords {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "[{}% {}%]", self.x * 100.0, self.y * 100.0)
  }
}

impl WGSLRepr for PTexCoords {
  type Repr = Vec2;

  fn convert(&self) -> Self::Repr {
    Vec2::new(self.x, self.y)
  }
}
