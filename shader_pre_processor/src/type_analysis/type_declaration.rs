use crate::type_analysis::member::Member;
use crate::type_analysis::named_type::{NamedType, NamedTypeParent};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TypeDeclaration {
  parent: NamedTypeParent,
  pub members: Vec<Member<String>>,
}

impl TypeDeclaration {
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

  pub fn with_member<T>(mut self, member: Member<T>) -> Self
  where
    T: ToString,
  {
    self.members.push(member.convert(|s| s.to_string()));
    self
  }

  pub fn add_member<T>(&mut self, member: Member<T>)
  where
    T: ToString,
  {
    self.members.push(member.convert(|s| s.to_string()))
  }
}

impl Deref for TypeDeclaration {
  type Target = NamedTypeParent;

  fn deref(&self) -> &Self::Target {
    &self.parent
  }
}

impl DerefMut for TypeDeclaration {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.parent
  }
}

impl NamedType for TypeDeclaration {
  fn name(&self) -> &str {
    self.parent.name()
  }

  fn rust_equivalent(&self) -> Option<&str> {
    self.parent.rust_equivalent()
  }
}

impl Display for TypeDeclaration {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} {{{}}}",
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
