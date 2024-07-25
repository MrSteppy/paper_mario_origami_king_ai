//Texture: TexRect, Square
//Triangle: 3x Clip, Color
//Line: 2x Clip, Color
//Pixel: 1x Clip, Color
//Circle: Square, Size, CircleCenter, TexCoords, degrees: f32..f32, Color
//Ring: Square, TexCoords, TexCoords..TexCoords, degrees: f32..f32, Color

// Pixel:2xu32
// Size:2xu32
// PTexCoords:2xf32
// Rect:2xPixel
// CircleCenter:PTexCoords|Pixel
// TexCoords:PTexCoords|Size+Pixel
// TexRect:2xTexCoords|Size+Rect
// Clip:4xf32|TexCoords
// Square:3xClip|TexRect

//CPU conversions:
// Square / TexCoords... => Clip...
// Square / TexRect => Square
// Size / Pixel... => TexCoords...
// Size / Rect => TexRect

//TODO cpu conversions

use std::fmt::{Debug, Display, Formatter};
use std::ops::Add;

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

impl Display for CircleCenter {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      CircleCenter::PTexCoords(p_tex_coords) => Display::fmt(p_tex_coords, f),
      CircleCenter::Pixel(pixel) => Display::fmt(pixel, f),
    }
  }
}

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
  Raw { x: f32, y: f32, z: f32, w: f32 },
  Screen(TexCoords),
}

impl Default for Clip {
  fn default() -> Self {
    Self::new(0.0, 0.0, 0.0)
  }
}

impl From<TexCoords> for Clip {
  fn from(value: TexCoords) -> Self {
    Self::Screen(value)
  }
}

impl Clip {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self::with_w(x, y, z, 1.0)
  }

  pub fn with_w(x: f32, y: f32, z: f32, w: f32) -> Self {
    Self::Raw { x, y, z, w }
  }

  pub fn screen<T>(tex_coords: T) -> Self
  where
    T: Into<TexCoords>,
  {
    Self::Screen(tex_coords.into())
  }
}

impl Display for Clip {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Clip::Raw { x, y, z, w } => write!(f, "({}, {}, {}, {})", x, y, z, w),
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
