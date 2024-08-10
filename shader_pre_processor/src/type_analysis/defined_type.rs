use crate::type_analysis::composite_type::CompositeType;
use crate::type_analysis::named_type::NamedType;
use crate::type_analysis::primitive_type::PrimitiveType;
use std::fmt::{Display, Formatter};
use std::iter::once;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DefinedType {
  Primitive(PrimitiveType),
  Composite(CompositeType),
}

impl From<PrimitiveType> for DefinedType {
  fn from(value: PrimitiveType) -> Self {
    Self::Primitive(value)
  }
}

impl From<CompositeType> for DefinedType {
  fn from(value: CompositeType) -> Self {
    Self::Composite(value)
  }
}

impl DefinedType {
  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    match self {
      Self::Primitive(primitive) => {
        Box::new(once(primitive)) as Box<dyn Iterator<Item = &PrimitiveType>>
      }
      Self::Composite(composite) => Box::new(composite.primitive_iter()),
    }
  }
}

impl NamedType for DefinedType {
  fn name(&self) -> &str {
    match self {
      DefinedType::Primitive(primitive) => primitive.name(),
      DefinedType::Composite(composite) => composite.name(),
    }
  }

  fn rust_equivalent(&self) -> Option<&str> {
    match self {
      DefinedType::Primitive(primitive) => primitive.rust_equivalent(),
      DefinedType::Composite(composite) => composite.rust_equivalent(),
    }
  }
}

impl Display for DefinedType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Primitive(primitive) => Display::fmt(primitive, f),
      Self::Composite(composite) => Display::fmt(composite, f),
    }
  }
}