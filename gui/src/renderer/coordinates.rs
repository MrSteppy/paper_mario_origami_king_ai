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
use std::ops::{Add, Div, Mul, Sub};

use glam::{Vec3, Vec4};

impl Div<TexCoords> for Square {
  type Output = Clip;

  fn div(self, rhs: TexCoords) -> Self::Output {
    let p_tex = rhs.as_p_tex_coords();
    let x_percent = clip_to_percent(p_tex.x);
    let y_percent = clip_to_percent(p_tex.y);

    let [top_left, bottom_left, bottom_right] = self.as_array().map(|clip| clip.as_p_clip());

    let x_direction = bottom_right - bottom_left;
    let y_direction = bottom_left - top_left;

    (top_left + x_percent * x_direction + y_percent * y_direction).into()
  }
}

fn clip_to_percent(clip: f32) -> f32 {
  (clip + 1.0) / 2.0
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

/// A point in clip coordinate space.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct PClip {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

impl From<Vec4> for PClip {
  fn from(value: Vec4) -> Self {
    Self {
      x: value.x,
      y: value.y,
      z: value.z,
      w: value.w,
    }
  }
}

impl From<Vec3> for PClip {
  fn from(value: Vec3) -> Self {
    Self::new(value.x, value.y, value.z)
  }
}

impl PClip {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z, w: 1.0 }
  }

  pub fn xyz(&self) -> Vec3 {
    Vec3::new(self.x, self.y, self.z)
  }
}

impl From<PClip> for Vec4 {
  fn from(value: PClip) -> Self {
    Self::new(value.x, value.y, value.z, value.w)
  }
}

impl Add for PClip {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    (self.xyz() * rhs.w + rhs.xyz() * self.w).extend(self.w * rhs.w).into()
  }
}

impl Sub for PClip {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    self + -1.0 * rhs
  }
}

impl Mul<f32> for PClip {
  type Output = Self;

  fn mul(self, rhs: f32) -> Self::Output {
    (self.xyz() * rhs).extend(self.w).into()
  }
}

impl Mul<PClip> for f32 {
  type Output = PClip;

  fn mul(self, rhs: PClip) -> Self::Output {
    rhs * self
  }
}

impl Div<f32> for PClip {
  type Output = Self;

  fn div(self, rhs: f32) -> Self::Output {
    self * (1.0 / rhs)
  }
}

impl Display for PClip {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
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
      TexCoords::Absolute { size, pixel } => (size / pixel).as_p_tex_coords()
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
      Clip::Screen(tex_coords) => (Square::default() / tex_coords).as_p_clip()
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
