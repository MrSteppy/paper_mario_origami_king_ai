use crate::primitive_composition::{ConversionError, PrimitiveComposition, TypeNameResolver};
use crate::struct_definition::StructDefinition;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum StructLayout {
  Simple(StructDefinition),
  Detailed {
    composition: PrimitiveComposition,
    generated_representation: Option<ReprInfo>,
  },
}

impl From<StructDefinition> for StructLayout {
  fn from(value: StructDefinition) -> Self {
    Self::Simple(value)
  }
}

impl From<PrimitiveComposition> for StructLayout {
  fn from(value: PrimitiveComposition) -> Self {
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
  ) -> Result<&mut PrimitiveComposition, ConversionError>
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
