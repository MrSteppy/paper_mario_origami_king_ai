use std::fmt::{Display, Formatter};

use crate::primitive_composition::primitive_type::PrimitiveType;
use crate::primitive_composition::PrimitiveComposition;
use crate::write_member;

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

  pub fn with_member<M>(mut self, member: M) -> Self
  where
    M: Into<Member>,
  {
    self.members.push(member.into());
    self
  }

  pub fn add<M>(&mut self, member: M) -> &mut Self
  where
    M: Into<Member>,
  {
    self.members.push(member.into());
    self
  }

  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    self
      .members
      .iter()
      .flat_map(|member| member.r#type.primitive_iter())
  }
}

impl Display for CompositeType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} ({})",
      self.name,
      self
        .members
        .iter()
        .map(|member| member.to_string())
        .collect::<Vec<_>>()
        .join(", ")
    )
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

impl Display for Member {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write_member(f, &self.annotation_values, &self.name, &self.r#type)
  }
}

#[cfg(test)]
mod test_composite_type {
  use crate::primitive_composition::composite_type::{CompositeType, Member};
  use crate::primitive_composition::primitive_type::PrimitiveType;

  #[test]
  fn test_iter() {
    let number_type = PrimitiveType::new("f32", 4, "f32");
    let pixel_type = CompositeType::new("Pixel")
      .with_member(Member::new("x", number_type.clone()))
      .with_member(Member::new("y", number_type.clone()));
    let str_type = PrimitiveType::new("str", 64, "String");
    let named_pixel_type = CompositeType::new("NamedPixel")
      .with_member(Member::new("name", str_type.clone()))
      .with_member(Member::new("pixel", pixel_type));

    let mut iter = named_pixel_type.primitive_iter();
    assert_eq!(Some(&str_type), iter.next());
    assert_eq!(Some(&number_type), iter.next());
    assert_eq!(Some(&number_type), iter.next());
    assert_eq!(None, iter.next());
  }
}
