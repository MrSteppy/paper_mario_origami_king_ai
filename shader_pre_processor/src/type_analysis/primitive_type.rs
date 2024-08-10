use crate::type_analysis::named_type::NamedType;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrimitiveType {
  pub name: String,
  pub rust_equivalent: String,
  pub size_in_alignments: usize,
  pub alignment_power: u8,
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
      alignment_power: u8::try_from(alignment.ilog2())
        .map_err(|_| PrimitiveTypeCreationError::AlignmentTooBig)?,
      rust_equivalent: rust_equivalent.to_string(),
    })
  }

  #[inline]
  pub const fn alignment(&self) -> usize {
    1 << self.alignment_power
  }

  #[inline]
  pub const fn size(&self) -> usize {
    self.alignment() * self.size_in_alignments
  }
}

impl NamedType for PrimitiveType {
  fn name(&self) -> &str {
    &self.name
  }

  fn rust_equivalent(&self) -> Option<&str> {
    Some(&self.rust_equivalent)
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
        format!("x{}", self.alignment())
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
  AlignmentTooBig,
}

impl Display for PrimitiveTypeCreationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PrimitiveTypeCreationError::InvalidAlignment => "Alignment has to be a power of 2",
        PrimitiveTypeCreationError::InvalidSize => "Size has to be a multiple of alignment",
        PrimitiveTypeCreationError::AlignmentTooBig => "Alignment is too big",
      }
    )
  }
}

impl Error for PrimitiveTypeCreationError {}

#[cfg(test)]
mod test_primitive_type {
  use crate::type_analysis::primitive_type::PrimitiveType;

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