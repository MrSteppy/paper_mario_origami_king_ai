use crate::primitive_composition::{ConversionError, PrimitiveComposition};
use crate::type_analysis::defined_type::DefinedType;
use crate::type_analysis::named_type::NamedType;
use crate::type_analysis::type_declaration::TypeDeclaration;
use crate::type_analysis::TypeNameResolver;

///deprecated: Use [`crate::type_analysis::declared_type::DeclaredType`]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[deprecated]
pub enum StructLayout {
  Simple(TypeDeclaration),
  Detailed {
    composition: DefinedType,
    generated_representation: Option<ReprInfo>,
  },
}

impl From<TypeDeclaration> for StructLayout {
  fn from(value: TypeDeclaration) -> Self {
    Self::Simple(value)
  }
}

impl From<DefinedType> for StructLayout {
  fn from(value: DefinedType) -> Self {
    Self::Detailed {
      composition: value,
      generated_representation: None,
    }
  }
}

impl StructLayout {
  pub fn name(&self) -> &str {
    match self {
      StructLayout::Simple(definition) => &definition.name,
      StructLayout::Detailed { composition, .. } => composition.name(),
    }
  }

  #[inline]
  pub fn create_primitive_composition<T>(
    &mut self,
    resolver: &mut T,
  ) -> Result<&mut DefinedType, ConversionError>
  where
    T: TypeNameResolver,
  {
    match self {
      StructLayout::Simple(struct_definition) => {
        let composition =
          PrimitiveComposition::from_struct_definition(struct_definition, resolver)?;
        *self = Self::Detailed {
          composition,
          generated_representation: None,
        };
        self.create_primitive_composition(resolver)
      }
      StructLayout::Detailed { composition, .. } => Ok(composition),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ReprInfo {
  pub name: String,
}

impl ReprInfo {
  pub fn new<S>(name: S) -> Self
  where
    S: ToString,
  {
    Self {
      name: name.to_string(),
    }
  }
}
