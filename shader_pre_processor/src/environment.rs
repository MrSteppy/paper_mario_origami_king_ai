use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::iter::once;

///Describes a type which can be directly converted from wgsl to wgpu, like `f32` or `vec4<f32>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrimitiveType {
  pub name: String,
  ///the size of this in terms of alignments.
  /// To get the actual size use method [`Self::size`]
  pub size_in_alignments: usize,
  ///the power of two to how many bytes this is aligned
  pub alignment_power: usize,
}

impl PrimitiveType {
  pub fn new<S>(name: S, size: usize) -> Self
  where
    S: ToString,
  {
    Self {
      name: name.to_string(),
      size_in_alignments: size,
      alignment_power: 0,
    }
  }

  pub fn new_aligned<S>(
    name: S,
    size: usize,
    alignment: usize,
  ) -> Result<Self, PrimitiveTypeCreationError>
  where
    S: ToString,
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

///A type which is composed of multiple other types
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CompositeType {
  pub name: String,
  pub members: Vec<Member>,
}

impl CompositeType {
  pub fn new<S>(name: S) -> Self
  where
    S: ToString,
  {
    Self {
      name: name.to_string(),
      members: vec![],
    }
  }

  pub fn add<M>(&mut self, member: M)
  where
    M: Into<Member>,
  {
    self.members.push(member.into());
  }

  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    self
      .members
      .iter()
      .flat_map(|member| member.r#type.primitive_iter())
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Member {
  pub name: String,
  pub r#type: PrimitiveComposition,
  ///The values of all annotations present, without the leading `@`
  pub annotation_values: Vec<String>,
}

impl Member {
  pub fn new<S, T>(name: S, r#type: T) -> Self
  where
    S: ToString,
    T: Into<PrimitiveComposition>,
  {
    Self {
      name: name.to_string(),
      r#type: r#type.into(),
      annotation_values: vec![],
    }
  }

  pub fn new_annotated<A, S, T>(annotations: &[A], name: S, r#type: T) -> Self
  where
    A: ToString,
    S: ToString,
    T: Into<PrimitiveComposition>,
  {
    Self {
      name: name.to_string(),
      r#type: r#type.into(),
      annotation_values: annotations.iter().map(|v| v.to_string()).collect(),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PrimitiveComposition {
  Primitive(PrimitiveType),
  Composite(CompositeType),
}

impl PrimitiveComposition {
  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    match self {
      PrimitiveComposition::Primitive(primitive) => {
        Box::new(once(primitive)) as Box<dyn Iterator<Item = &PrimitiveType>>
      }
      PrimitiveComposition::Composite(composite) => Box::new(composite.primitive_iter()),
    }
  }

  //TODO create memory layout
}

impl From<PrimitiveType> for PrimitiveComposition {
  fn from(value: PrimitiveType) -> Self {
    Self::Primitive(value)
  }
}

impl From<CompositeType> for PrimitiveComposition {
  fn from(value: CompositeType) -> Self {
    Self::Composite(value)
  }
}

#[cfg(test)]
mod test_primitive_type {
  use crate::environment::PrimitiveType;

  #[test]
  fn test_size() {
    assert_eq!(4, PrimitiveType::new("f32", 4).size());
    assert_eq!(16, PrimitiveType::new_aligned("vec4<f32>", 16, 16).unwrap().size());
    assert_eq!(16, PrimitiveType {
      name: "vec2<f64>".to_string(),
      size_in_alignments: 2,
      alignment_power: 3,
    }.size());
  }
}

//TODO unit tests for CompositeType
