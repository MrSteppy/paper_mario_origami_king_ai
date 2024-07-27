use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Sub};

use glam::{Vec3, Vec4};

use crate::renderer::coordinates::WGSLRepr;

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

impl Add for PClip {
  type Output = Self;

  fn add(self, rhs: Self) -> Self::Output {
    (self.xyz() * rhs.w + rhs.xyz() * self.w)
      .extend(self.w * rhs.w)
      .into()
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

impl WGSLRepr for PClip {
  type Repr = Vec4;

  fn convert(&self) -> Self::Repr {
    Vec4::new(self.x, self.y, self.z, self.w)
  }
}
