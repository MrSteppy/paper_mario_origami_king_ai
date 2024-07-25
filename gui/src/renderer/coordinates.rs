//Texture: 2x TexClip, 3x Clip
//Triangle: 3x Clip, Color
//Line: 2x Clip, Color
//Pixel: 1x Clip, Color
//Circle: Area, PixelSize, TexClip, radius: f32, degrees: f32..f32, Color
//Ring: Area, TexClip, max_radius: f32, min_radius: f32, degrees: f32..f32, Color

use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, BitOr, Deref, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use glam::Vec4;

///Describes the absolute size of a texture in pixels
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PixelSize {
  pub width: u32,
  pub height: u32,
}

impl PixelSize {
  pub fn new(width: u32, height: u32) -> Self {
    Self { width, height }
  }

  pub fn convert_pixel(&self, pixel: Pixel) -> TexClip {
    TexClip::new(
      percent_to_clip(pixel.x as f32 / self.width as f32),
      percent_to_clip(pixel.y as f32 / self.height as f32),
    )
  }
}

impl BitOr<Pixel> for PixelSize {
  type Output = TexClip;

  fn bitor(self, rhs: Pixel) -> Self::Output {
    self.convert_pixel(rhs)
  }
}

impl<const N: usize> BitOr<[Pixel; N]> for PixelSize {
  type Output = [TexClip; N];

  fn bitor(self, rhs: [Pixel; N]) -> Self::Output {
    rhs.map(|pixel| self.convert_pixel(pixel))
  }
}

fn percent_to_clip(percent: f32) -> f32 {
  2.0 * percent - 1.0
}

///Denotes a pixel, where (0, 0) is the one in the top left corner
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

///two [`TexClip`]s describing the top left and bottom right corners of a square on a texture
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TexArea {
  pub top_left_corner: TexClip,
  pub bottom_right_corner: TexClip,
}

impl Default for TexArea {
  fn default() -> Self {
    Self::from([TexClip::new(-1.0, 1.0), TexClip::new(1.0, -1.0)])
  }
}

impl From<[TexClip; 2]> for TexArea {
  fn from(value: [TexClip; 2]) -> Self {
    Self {
      top_left_corner: value[0],
      bottom_right_corner: value[1],
    }
  }
}

impl From<TexArea> for [TexClip; 2] {
  fn from(value: TexArea) -> Self {
    [value.top_left_corner, value.bottom_right_corner]
  }
}

impl TexArea {
  pub fn bottom_left_corner(&self) -> TexClip {
    TexClip::new(
      self.top_left_corner.x / self.top_left_corner.w,
      self.bottom_right_corner.y / self.bottom_right_corner.w,
    )
  }
}

///two-dimensional [`Clip`]s, for a texture
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TexClip {
  pub x: f32,
  pub y: f32,
  pub w: f32,
}

impl Default for TexClip {
  fn default() -> Self {
    Self {
      x: 0.0,
      y: 0.0,
      w: 1.0,
    }
  }
}

impl TexClip {
  pub fn new(x: f32, y: f32) -> Self {
    Self { x, y, w: 1.0 }
  }
}

///A hyper area, described by three of its corners: top left, bottom left, bottom right. The fourth
/// corner can be obtained by mirroring the described triangle
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ClipArea {
  pub top_left_corner: Clip,
  pub bottom_left_corner: Clip,
  pub bottom_right_corner: Clip,
}

impl Default for ClipArea {
  fn default() -> Self {
    Self::from([
      Clip::new(-1.0, 1.0, 0.0),
      Clip::new(-1.0, -1.0, 0.0),
      Clip::new(1.0, -1.0, 0.0),
    ])
  }
}

impl From<[Clip; 3]> for ClipArea {
  fn from(value: [Clip; 3]) -> Self {
    Self {
      top_left_corner: value[0],
      bottom_left_corner: value[1],
      bottom_right_corner: value[2],
    }
  }
}

impl ClipArea {
  pub fn top_right_corner(&self) -> Clip {
    self.top_left_corner + self.bottom_right_corner - self.bottom_left_corner
  }

  pub fn apply_tex_clip(&self, tex_clip: TexClip) -> Clip {
    let x_percent = clip_to_percent(tex_clip.x / tex_clip.w);
    let y_percent = clip_to_percent(tex_clip.y / tex_clip.w);

    let x_dir = (self.bottom_right_corner - self.bottom_left_corner).adjust_w(1.0);
    let y_dir = (self.bottom_left_corner - self.top_left_corner).adjust_w(1.0);

    self.top_left_corner + x_dir * x_percent + y_dir * y_percent
  }

  pub fn sub_area(&self, tex_area: TexArea) -> Self {
    (*self
      | [
        tex_area.top_left_corner,
        tex_area.bottom_left_corner(),
        tex_area.bottom_right_corner,
      ])
    .into()
  }
}

impl BitOr<TexClip> for ClipArea {
  type Output = Clip;

  fn bitor(self, rhs: TexClip) -> Self::Output {
    self.apply_tex_clip(rhs)
  }
}

impl<const N: usize> BitOr<[TexClip; N]> for ClipArea {
  type Output = [Clip; N];

  fn bitor(self, rhs: [TexClip; N]) -> Self::Output {
    rhs.map(|tex_clip| self.apply_tex_clip(tex_clip))
  }
}

impl BitOr<TexArea> for ClipArea {
  type Output = Self;

  fn bitor(self, rhs: TexArea) -> Self::Output {
    self.sub_area(rhs)
  }
}

impl BitOr<PixelSize> for ClipArea {
  type Output = SizedClipArea;

  fn bitor(self, rhs: PixelSize) -> Self::Output {
    SizedClipArea {
      area: self,
      size: rhs,
    }
  }
}

fn clip_to_percent(clip: f32) -> f32 {
  (clip + 1.0) / 2.0
}

///Represents clip coordinates: x, y, z and a scale w.
/// In order to be visible on a normal screen, `x / w` and `y / w` must be in `-1.0..1.0` and
/// `z / w` must be in `0.0..1.0`
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Clip {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

impl Default for Clip {
  fn default() -> Self {
    Self {
      x: 0.0,
      y: 0.0,
      z: 0.0,
      w: 1.0,
    }
  }
}

impl Clip {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self { x, y, z, w: 1.0 }
  }

  pub fn adjust_w(self, w: f32) -> Self {
    let mut adjusted = self * (w / self.w);
    adjusted.w = w;
    adjusted
  }
}

impl From<Vec4> for Clip {
  fn from(value: Vec4) -> Self {
    Self {
      x: value.x,
      y: value.y,
      z: value.z,
      w: value.w,
    }
  }
}

impl From<Clip> for Vec4 {
  fn from(value: Clip) -> Self {
    Vec4::new(value.x, value.y, value.z, value.w)
  }
}

impl Mul<f32> for Clip {
  type Output = Self;

  fn mul(mut self, rhs: f32) -> Self::Output {
    self.x *= rhs;
    self.y *= rhs;
    self.z *= rhs;
    self
  }
}

impl Mul<Clip> for f32 {
  type Output = Clip;

  fn mul(self, rhs: Clip) -> Self::Output {
    rhs * self
  }
}

impl MulAssign<f32> for Clip {
  fn mul_assign(&mut self, rhs: f32) {
    *self = *self * rhs
  }
}

impl Div<f32> for Clip {
  type Output = Self;

  fn div(mut self, rhs: f32) -> Self::Output {
    self.x /= rhs;
    self.y /= rhs;
    self.z /= rhs;
    self
  }
}

impl DivAssign<f32> for Clip {
  fn div_assign(&mut self, rhs: f32) {
    *self = *self / rhs
  }
}

impl Add for Clip {
  type Output = Self;

  fn add(mut self, mut rhs: Self) -> Self::Output {
    self *= rhs.w;
    rhs *= self.w;
    self.x += rhs.x;
    self.y += rhs.y;
    self.z += rhs.z;
    self.w *= rhs.w;
    self
  }
}

impl AddAssign for Clip {
  fn add_assign(&mut self, rhs: Self) {
    *self = *self + rhs
  }
}

impl Sub for Clip {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self::Output {
    self + -1.0 * rhs
  }
}

impl SubAssign for Clip {
  fn sub_assign(&mut self, rhs: Self) {
    *self = *self - rhs
  }
}

impl Display for Clip {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
  }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct SizedClipArea {
  pub area: ClipArea,
  pub size: PixelSize,
}

impl Deref for SizedClipArea {
  type Target = ClipArea;

  fn deref(&self) -> &Self::Target {
    &self.area
  }
}

impl BitOr<Pixel> for SizedClipArea {
  type Output = Clip;

  fn bitor(self, rhs: Pixel) -> Self::Output {
    self.area | (self.size | rhs)
  }
}

impl<const N: usize> BitOr<[Pixel; N]> for SizedClipArea {
  type Output = [Clip; N];

  fn bitor(self, rhs: [Pixel; N]) -> Self::Output {
    self.area | (self.size | rhs)
  }
}

#[cfg(test)]
mod test {
  use crate::renderer::coordinates::clip_to_percent;

  #[test]
  fn test_clip_to_percent() {
    assert_eq!(0.5, clip_to_percent(0.0));
    assert_eq!(0.0, clip_to_percent(-1.0));
    assert_eq!(1.0, clip_to_percent(1.0));
  }
}

#[cfg(test)]
mod test_conversions {
  use crate::renderer::coordinates::{Clip, ClipArea, Pixel, PixelSize};

  #[test]
  fn test_pixel_to_clips() {
    let pixel1 = Pixel::new(4, 4);
    let pixel2 = Pixel::new(16, 8);
    let size = PixelSize::new(32, 16);

    let clip1 = Clip::new(-0.75, 0.5, 0.0);
    let clip2 = Clip::new(0.0, 0.0, 0.0);

    let area = ClipArea::default();

    assert_eq!([clip1, clip2], area | size | [pixel1, pixel2]);
  }
}

#[cfg(test)]
mod test_clip_area {
  use crate::renderer::coordinates::{Clip, ClipArea, TexClip};

  #[test]
  fn test_apply_tex_clip_to_square() {
    let area = ClipArea::default();
    let tex_clip = TexClip::new(1.0, 0.0);
    assert_eq!(Clip::new(1.0, 0.0, 0.0), area | tex_clip);
  }
}
