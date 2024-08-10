use crate::type_analysis::composite_type::CompositeType;
use crate::type_analysis::defined_type::DefinedType;
use crate::type_analysis::named_type::NamedType;
use crate::type_analysis::primitive_type::PrimitiveType;
use crate::type_analysis::type_declaration::TypeDeclaration;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DeclaredType {
  Declared(TypeDeclaration),
  Defined(DefinedType),
}

impl From<TypeDeclaration> for DeclaredType {
  fn from(value: TypeDeclaration) -> Self {
    Self::Declared(value)
  }
}

impl From<DefinedType> for DeclaredType {
  fn from(value: DefinedType) -> Self {
    Self::Defined(value)
  }
}

impl From<PrimitiveType> for DeclaredType {
  fn from(value: PrimitiveType) -> Self {
    Self::Defined(value.into())
  }
}

impl From<CompositeType> for DeclaredType {
  fn from(value: CompositeType) -> Self {
    Self::Defined(value.into())
  }
}

impl NamedType for DeclaredType {
  fn name(&self) -> &str {
    match self {
      DeclaredType::Declared(declaration) => declaration.name(),
      DeclaredType::Defined(definition) => definition.name(),
    }
  }

  fn rust_equivalent(&self) -> Option<&str> {
    match self {
      DeclaredType::Declared(declaration) => declaration.rust_equivalent(),
      DeclaredType::Defined(definition) => definition.rust_equivalent(),
    }
  }
}

impl Display for DeclaredType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      DeclaredType::Declared(declaration) => Display::fmt(declaration, f),
      DeclaredType::Defined(definition) => Display::fmt(definition, f),
    }
  }
}