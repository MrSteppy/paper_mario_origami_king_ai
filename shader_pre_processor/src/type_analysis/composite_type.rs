use crate::type_analysis::defined_type::DefinedType;
use crate::type_analysis::member::Member;
use crate::type_analysis::named_type::{NamedType, NamedTypeParent};
use crate::type_analysis::primitive_type::PrimitiveType;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CompositeType {
  parent: NamedTypeParent,
  pub members: Vec<Member<DefinedType>>,
}

impl CompositeType {
  pub fn new<S>(name: S) -> Self
  where
    S: ToString,
  {
    Self {
      parent: NamedTypeParent::new(name),
      members: vec![],
    }
  }

  pub fn with_rust_equivalent<R>(mut self, rust_equivalent: R) -> Self
  where
    R: ToString,
  {
    self.parent = self.parent.with_rust_equivalent(rust_equivalent);
    self
  }

  pub fn with_member<R>(mut self, member: Member<R>) -> Self
  where
    R: Into<DefinedType>,
  {
    self.members.push(member.convert_into());
    self
  }

  pub fn add_member<R>(&mut self, member: Member<R>)
  where
    R: Into<DefinedType>,
  {
    self.members.push(member.convert_into());
  }

  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    self
      .members
      .iter()
      .flat_map(|member| member.r#type.primitive_iter())
  }
}

impl Deref for CompositeType {
  type Target = NamedTypeParent;

  fn deref(&self) -> &Self::Target {
    &self.parent
  }
}

impl DerefMut for CompositeType {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.parent
  }
}

impl NamedType for CompositeType {
  fn name(&self) -> &str {
    self.parent.name()
  }

  fn rust_equivalent(&self) -> Option<&str> {
    self.parent.rust_equivalent()
  }
}

impl Display for CompositeType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} {{{}}})",
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

#[cfg(test)]
mod test {
  use crate::type_analysis::composite_type::CompositeType;
  use crate::type_analysis::member::Member;
  use crate::type_analysis::primitive_type::PrimitiveType;

  #[test]
  fn test_primitive_iter() {
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