use crate::primitive_composition::PrimitiveComposition;
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
