use std::fmt::{Display, Formatter};

use crate::renderer::coordinates::FloatArrayRepr;

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

impl FloatArrayRepr for Pixel {
  const N: usize = 2;

  fn to_float_array(self) -> [f32; Self::N] {
    [self.x as f32, self.y as f32]
  }
}
