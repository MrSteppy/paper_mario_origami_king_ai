use std::fmt::{Display, Formatter};

use glam::UVec4;

use crate::renderer::coordinates::pixel::Pixel;
use crate::renderer::coordinates::WGSLRepr;

///A rectangle described by two [`Pixel`]s which denote the top left corner and the bottom right one
/// of the rectangle
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Rect {
  pub top_left: Pixel,
  pub bottom_right: Pixel,
}

impl From<[Pixel; 2]> for Rect {
  fn from(value: [Pixel; 2]) -> Self {
    Self {
      top_left: value[0],
      bottom_right: value[1],
    }
  }
}

impl Rect {
  pub fn new(top_left: Pixel, bottom_right: Pixel) -> Self {
    Self {
      top_left,
      bottom_right,
    }
  }
}

impl Display for Rect {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}->{}", self.top_left, self.bottom_right)
  }
}

impl WGSLRepr for Rect {
  type Repr = UVec4;

  fn to_wgsl_repr(self) -> Self::Repr {
    UVec4::from((
      self.top_left.to_wgsl_repr(),
      self.bottom_right.to_wgsl_repr(),
    ))
  }
}
