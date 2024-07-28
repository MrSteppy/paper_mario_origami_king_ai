//Texture: TexRect, Square
//Triangle: 3x Clip, Color
//Line: 2x Clip, Color
//Pixel: 1x Clip, Color
//Circle: Square, Size, CircleCenter, TexCoords, degrees: f32..f32, Color
//Ring: Square, TexCoords, TexCoords..TexCoords, degrees: f32..f32, Color

// Pixel:2xu32
// Size:2xu32
// PTexCoords:2xf32
// PClip:4xf32
// Rect:2xPixel
// CircleCenter:PTexCoords|Pixel
// TexCoords:PTexCoords|Size+Pixel
// TexRect:2xTexCoords|Size+Rect
// Clip:PClip|TexCoords
// Square:3xClip|TexRect

//CPU conversions:
// Square / TexCoords... => Clip...
// Square / TexRect => Square
// Size / Pixel... => TexCoords...
// Size / Rect => TexRect

use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Div};

use p_clip::PClip;
use p_tex_coords::PTexCoords;
use pixel::Pixel;
use rect::Rect;
use size::Size;

mod circle_center;
mod p_clip;
mod p_tex_coords;
mod pixel;
mod rect;
mod size;

impl Div<TexCoords> for Square {
  type Output = Clip;

  fn div(self, rhs: TexCoords) -> Self::Output {
    let p_tex = rhs.as_p_tex_coords();

    let [top_left, bottom_left, bottom_right] = self.as_array().map(|clip| clip.as_p_clip());

    let x_direction = bottom_right - bottom_left;
    let y_direction = bottom_left - top_left;

    (top_left + p_tex.x * x_direction + p_tex.y * y_direction).into()
  }
}

impl<const N: usize> Div<[TexCoords; N]> for Square {
  type Output = [Clip; N];

  fn div(self, rhs: [TexCoords; N]) -> Self::Output {
    rhs.map(|tex_coords| self / tex_coords)
  }
}

impl Div<TexRect> for Square {
  type Output = Square;

  fn div(self, rhs: TexRect) -> Self::Output {
    let [top_left, bottom_right] = rhs.into();
    let bottom_left = TexCoords::new(
      PTexCoords::from(top_left).x,
      PTexCoords::from(bottom_right).y,
    );
    (self / [top_left, bottom_left, bottom_right]).into()
  }
}

impl Div<Pixel> for Size {
  type Output = TexCoords;

  fn div(self, rhs: Pixel) -> Self::Output {
    TexCoords::new(
      self.width as f32 / rhs.x as f32,
      self.height as f32 / rhs.y as f32,
    )
  }
}

impl<const N: usize> Div<[Pixel; N]> for Size {
  type Output = [TexCoords; N];

  fn div(self, rhs: [Pixel; N]) -> Self::Output {
    rhs.map(|pixel| self / pixel)
  }
}

impl Div<Rect> for Size {
  type Output = TexRect;

  fn div(self, rhs: Rect) -> Self::Output {
    TexRect::new(self / rhs.top_left, self / rhs.bottom_right)
  }
}

//coordinates

///Denote a pixel on a texture
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TexCoords {
  Relative(PTexCoords),
  Absolute {
    ///the size of the texture
    size: Size,
    ///the exact pixel
    pixel: Pixel,
  },
}

impl Default for TexCoords {
  fn default() -> Self {
    Self::Relative(PTexCoords::default())
  }
}

impl From<PTexCoords> for TexCoords {
  fn from(value: PTexCoords) -> Self {
    Self::Relative(value)
  }
}

impl TexCoords {
  pub fn new(x: f32, y: f32) -> Self {
    Self::from(PTexCoords::new(x, y))
  }

  #[inline]
  pub fn as_p_tex_coords(&self) -> PTexCoords {
    match *self {
      TexCoords::Relative(coords) => coords,
      TexCoords::Absolute { size, pixel } => (size / pixel).as_p_tex_coords(),
    }
  }
}

impl Add<Pixel> for Size {
  type Output = TexCoords;

  fn add(self, rhs: Pixel) -> Self::Output {
    TexCoords::Absolute {
      size: self,
      pixel: rhs,
    }
  }
}

impl From<TexCoords> for PTexCoords {
  fn from(value: TexCoords) -> Self {
    value.as_p_tex_coords()
  }
}

impl Display for TexCoords {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      TexCoords::Relative(p_tex_coords) => Display::fmt(p_tex_coords, f),
      TexCoords::Absolute { size, pixel } => {
        write!(f, "[{} / {}]", size, pixel)
      }
    }
  }
}

///Denote a rectangle of pixels on a texture
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TexRect {
  Relative {
    top_left: TexCoords,
    bottom_right: TexCoords,
  },
  Absolute {
    size: Size,
    rect: Rect,
  },
}

impl From<[TexCoords; 2]> for TexRect {
  fn from(value: [TexCoords; 2]) -> Self {
    Self::Relative {
      top_left: value[0],
      bottom_right: value[1],
    }
  }
}

impl TexRect {
  pub fn new<T, B>(top_left: T, bottom_right: B) -> Self
  where
    T: Into<TexCoords>,
    B: Into<TexCoords>,
  {
    Self::Relative {
      top_left: top_left.into(),
      bottom_right: bottom_right.into(),
    }
  }

  pub fn as_array(&self) -> [TexCoords; 2] {
    match *self {
      TexRect::Relative {
        top_left,
        bottom_right,
      } => [top_left, bottom_right],
      TexRect::Absolute { size, rect } => (size / rect).into(),
    }
  }
}

impl<R> Add<R> for Size
where
  R: Into<Rect>,
{
  type Output = TexRect;

  fn add(self, rhs: R) -> Self::Output {
    TexRect::Absolute {
      size: self,
      rect: rhs.into(),
    }
  }
}

impl From<TexRect> for [TexCoords; 2] {
  fn from(value: TexRect) -> Self {
    value.as_array()
  }
}

impl Display for TexRect {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      TexRect::Relative {
        top_left,
        bottom_right,
      } => write!(f, "{}->{}", top_left, bottom_right),
      TexRect::Absolute { size, rect } => write!(f, "{} / {}", size, rect),
    }
  }
}

///Describes a clip coordinate in 3d space. For kind `Screen` the clip
/// is relative to the area of `[-1.0, 1.0]->[1.0, -1.0]` in z = `0.0`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Clip {
  Raw(PClip),
  Screen(TexCoords),
}

impl Default for Clip {
  fn default() -> Self {
    Self::new(0.0, 0.0, 0.0)
  }
}

impl From<PClip> for Clip {
  fn from(value: PClip) -> Self {
    Self::Raw(value)
  }
}

impl From<TexCoords> for Clip {
  fn from(value: TexCoords) -> Self {
    Self::Screen(value)
  }
}

impl Clip {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self::from(PClip::new(x, y, z))
  }

  pub fn screen<T>(tex_coords: T) -> Self
  where
    T: Into<TexCoords>,
  {
    Self::Screen(tex_coords.into())
  }

  #[inline]
  pub fn as_p_clip(&self) -> PClip {
    match *self {
      Clip::Raw(inner) => inner,
      Clip::Screen(tex_coords) => (Square::default() / tex_coords).as_p_clip(),
    }
  }
}

impl From<Clip> for PClip {
  fn from(value: Clip) -> Self {
    value.as_p_clip()
  }
}

impl Display for Clip {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Clip::Raw(inner) => Display::fmt(inner, f),
      Clip::Screen(tex_coords) => write!(f, "(screen / {})", tex_coords),
    }
  }
}

///Denotes a square in 3d clip space. For kind `Screen` the square
/// is relative to the area of `[-1.0, 1.0]->[1.0, -1.0]` in z = `0.0`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Square {
  Span {
    top_left: Clip,
    bottom_left: Clip,
    bottom_right: Clip,
  },
  Screen(TexRect),
}

impl Default for Square {
  fn default() -> Self {
    Self::Span {
      top_left: Clip::new(-1.0, 1.0, 0.0),
      bottom_left: Clip::new(-1.0, -1.0, 0.0),
      bottom_right: Clip::new(1.0, -1.0, 0.0),
    }
  }
}

impl From<[Clip; 3]> for Square {
  fn from(value: [Clip; 3]) -> Self {
    Self::Span {
      top_left: value[0],
      bottom_left: value[1],
      bottom_right: value[2],
    }
  }
}

impl From<TexRect> for Square {
  fn from(value: TexRect) -> Self {
    Self::Screen(value)
  }
}

impl Square {
  pub fn new<T, R, S>(top_left: T, bottom_left: R, bottom_right: S) -> Self
  where
    T: Into<Clip>,
    R: Into<Clip>,
    S: Into<Clip>,
  {
    Self::Span {
      top_left: top_left.into(),
      bottom_left: bottom_left.into(),
      bottom_right: bottom_right.into(),
    }
  }

  pub fn as_array(&self) -> [Clip; 3] {
    match *self {
      Square::Span {
        top_left,
        bottom_left,
        bottom_right,
      } => [top_left, bottom_left, bottom_right],
      Square::Screen(tex_rect) => (Square::default() / tex_rect).into(),
    }
  }
}

impl From<Square> for [Clip; 3] {
  fn from(value: Square) -> Self {
    value.as_array()
  }
}

impl Display for Square {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Square::Span {
        top_left,
        bottom_left,
        bottom_right,
      } => write!(f, "{}->{}->{}", top_left, bottom_left, bottom_right),
      Square::Screen(tex_rect) => write!(f, "screen / {}", tex_rect),
    }
  }
}

#[deprecated]
pub trait FloatArrayRepr {
  fn to_float_array(self) -> Vec<f32>;
}
