use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};

///Describes a type which can be directly converted from wgsl to wgpu, like `f32` or `vec4<f32>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrimitiveType {
  pub name: String,
  ///the size of this in terms of alignments.
  /// To get the actual size use method [`Self::size`]
  pub size_in_alignments: usize,
  ///the power of two to how many bytes this is aligned
  pub alignment_power: usize,
  ///The fully qualified path to a rust equivalent type
  pub rust_equivalent: String,
}

impl PrimitiveType {
  pub fn new<S, E>(name: S, size: usize, rust_equivalent: E) -> Self
  where
    S: ToString,
    E: ToString,
  {
    Self {
      name: name.to_string(),
      size_in_alignments: size,
      alignment_power: 0,
      rust_equivalent: rust_equivalent.to_string(),
    }
  }

  pub fn new_aligned<S, E>(
    name: S,
    size: usize,
    alignment: usize,
    rust_equivalent: E,
  ) -> Result<Self, PrimitiveTypeCreationError>
  where
    S: ToString,
    E: ToString,
  {
    if !alignment.is_power_of_two() {
      return Err(PrimitiveTypeCreationError::InvalidAlignment);
    }
    if size % alignment != 0 {
      return Err(PrimitiveTypeCreationError::InvalidSize);
    }

    Ok(Self {
      name: name.to_string(),
      size_in_alignments: size / alignment,
      alignment_power: alignment.ilog2() as usize,
      rust_equivalent: rust_equivalent.to_string(),
    })
  }

  #[inline]
  pub const fn size(&self) -> usize {
    (1 << self.alignment_power) * self.size_in_alignments
  }
}

impl PartialOrd for PrimitiveType {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for PrimitiveType {
  fn cmp(&self, other: &Self) -> Ordering {
    self.size().cmp(&other.size())
  }
}

impl Display for PrimitiveType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}[{}{}byte]",
      self.name,
      self.size_in_alignments,
      if self.alignment_power > 0 {
        format!("x{}", self.alignment_power)
      } else {
        "".to_string()
      }
    )
  }
}

#[derive(Debug)]
pub enum PrimitiveTypeCreationError {
  InvalidAlignment,
  InvalidSize,
}

impl Display for PrimitiveTypeCreationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PrimitiveTypeCreationError::InvalidAlignment => "Alignment has to be a power of 2",
        PrimitiveTypeCreationError::InvalidSize => "Size has to be a multiple of alignment",
      }
    )
  }
}

impl Error for PrimitiveTypeCreationError {}

#[cfg(test)]
mod test_primitive_type {
  use crate::primitive_composition::primitive_type::PrimitiveType;

  #[test]
  fn test_size() {
    assert_eq!(4, PrimitiveType::new("f32", 4, "f32").size());
    assert_eq!(
      16,
      PrimitiveType::new_aligned("vec4<f32>", 16, 16, "glam::Vec4")
        .unwrap()
        .size()
    );
    assert_eq!(
      16,
      PrimitiveType {
        name: "vec2<f64>".to_string(),
        size_in_alignments: 2,
        alignment_power: 3,
        rust_equivalent: "glam::DVec2".to_string(),
      }
      .size()
    );
  }
}
