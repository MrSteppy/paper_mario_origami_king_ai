use std::fmt::{Display, Formatter};

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
