use std::fmt::{Display, Formatter};

use crate::renderer::coordinates::p_tex_coords::PTexCoords;
use crate::renderer::coordinates::pixel::Pixel;
use crate::renderer::coordinates::size::Size;
use crate::renderer::coordinates::TexCoords;

///Describes where the center of a circle is located
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CircleCenter {
  PTexCoords(PTexCoords),
  Pixel(Pixel),
}

impl Default for CircleCenter {
  fn default() -> Self {
    Self::PTexCoords(PTexCoords::new(0.5, 0.5))
  }
}

impl From<PTexCoords> for CircleCenter {
  fn from(value: PTexCoords) -> Self {
    Self::PTexCoords(value)
  }
}

impl From<Pixel> for CircleCenter {
  fn from(value: Pixel) -> Self {
    Self::Pixel(value)
  }
}

impl CircleCenter {
  pub fn as_tex_coords<S>(&self, size: S) -> TexCoords
  where
    S: Into<Size>,
  {
    match *self {
      CircleCenter::PTexCoords(coords) => coords.into(),
      CircleCenter::Pixel(pixel) => size.into() / pixel,
    }
  }
}

impl Display for CircleCenter {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CircleCenter::PTexCoords(p_tex_coords) => Display::fmt(p_tex_coords, f),
      CircleCenter::Pixel(pixel) => Display::fmt(pixel, f),
    }
  }
}
