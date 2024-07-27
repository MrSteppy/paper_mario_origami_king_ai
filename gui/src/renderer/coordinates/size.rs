use std::fmt::{Display, Formatter};

use glam::UVec2;

use crate::renderer::coordinates::WGSLRepr;

///Describes the size of a canvas, texture or square area
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Size {
  pub width: u32,
  pub height: u32,
}

impl Size {
  pub fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }
}

impl Display for Size {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}x{}", self.width, self.height)
  }
}

impl WGSLRepr for Size {
  type Repr = UVec2;

  fn convert(&self) -> Self::Repr {
    UVec2::new(self.width, self.height)
  }
}
