use std::iter::once;

use enum_assoc::Assoc;

use crate::StructDefinition;

//TODO better use a type resolver instead of the direct struct definitions
pub fn create_memory_layout(struct_definition: &StructDefinition, other_struct_definitions: &Vec<&StructDefinition>) -> MemoryLayout {
  todo!()
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MemoryLayout {
  Composed(CompositeLayout),
  Primitive(PrimitiveLayout),
}

impl Default for MemoryLayout {
  fn default() -> Self {
    Self::Primitive(Default::default())
  }
}

impl From<CompositeLayout> for MemoryLayout {
  fn from(value: CompositeLayout) -> Self {
    Self::Composed(value)
  }
}

impl From<PrimitiveLayout> for MemoryLayout {
  fn from(value: PrimitiveLayout) -> Self {
    Self::Primitive(value)
  }
}

impl MemoryLayoutCommon for MemoryLayout {
  fn iter(&self) -> impl Iterator<Item = &PrimitiveLayout> {
    match self {
      MemoryLayout::Composed(composed) => {
        Box::new(composed.iter()) as Box<dyn Iterator<Item = &PrimitiveLayout>>
      }
      MemoryLayout::Primitive(primitive) => Box::new(primitive.iter()),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CompositeLayout {
  pub type_name: String,
  pub components: Vec<MemoryLayout>,
}

impl CompositeLayout {
  pub fn new<S>(type_name: S) -> Self
  where
    S: ToString,
  {
    Self {
      type_name: type_name.to_string(),
      components: Default::default(),
    }
  }

  pub fn add<M>(&mut self, component: M)
  where
    M: Into<MemoryLayout>,
  {
    self.components.push(component.into())
  }
}

impl MemoryLayoutCommon for CompositeLayout {
  fn iter(&self) -> impl Iterator<Item = &PrimitiveLayout> {
    self
      .components
      .iter()
      .flat_map(|component| component.iter())
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct PrimitiveLayout {
  pub primitive_type: PrimitiveType,
}

impl MemoryLayoutCommon for PrimitiveLayout {
  fn iter(&self) -> impl Iterator<Item = &PrimitiveLayout> {
    once(self)
  }
}

pub trait MemoryLayoutCommon {
  fn iter(&self) -> impl Iterator<Item = &PrimitiveLayout>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default, Assoc)]
#[func(pub const fn name(&self) -> &'static str)]
#[func(pub fn from_name(name: &str) -> Option<Self>)]
pub enum PrimitiveType {
  #[default]
  #[assoc(name = "f32")]
  #[assoc(from_name = "f32")]
  F32,
  #[assoc(name = "u32")]
  #[assoc(from_name = "u32")]
  U32,
}
