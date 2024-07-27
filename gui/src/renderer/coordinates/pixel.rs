use std::fmt::{Display, Formatter};

use glam::UVec2;

use crate::renderer::coordinates::WGSLRepr;

///Denotes a pixel on a canvas or texture
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct Pixel {
  pub x: u32,
  pub y: u32,
}

impl Pixel {
  pub fn new(x: u32, y: u32) -> Self {
    Self { x, y }
  }
}

impl Display for Pixel {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "[{} {}]", self.x, self.y)
  }
}

impl WGSLRepr for Pixel {
  type Repr = UVec2;

  fn to_wgsl_repr(self) -> Self::Repr {
    UVec2::new(self.x, self.y)
  }
}
