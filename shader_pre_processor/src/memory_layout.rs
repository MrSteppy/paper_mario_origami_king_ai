use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::once;

use crate::struct_definition::StructDefinition;

pub type StructDefinitionCache = HashMap<String, (StructDefinition, Option<MemoryLayout>)>;

#[derive(Debug)]
pub struct TypeResolver<'a> {
  pub primitive_types: &'a HashSet<String>,
  pub struct_definition_cache: &'a mut StructDefinitionCache,
}

pub fn create_memory_layout(
  target_type_name: &str,
  type_resolver: &mut TypeResolver,
) -> Result<MemoryLayout, MemoryLayoutCreationError> {
  if type_resolver.primitive_types.contains(target_type_name) {
    return Ok(MemoryLayout::Primitive(PrimitiveLayout {
      primitive_type: target_type_name.to_string(),
    }));
  }

  if let Some((struct_definition, memory_layout)) =
    type_resolver.struct_definition_cache.get(target_type_name)
  {
    if let Some(memory_layout) = memory_layout {
      return Ok(memory_layout.clone());
    }

    let struct_definition = struct_definition.clone();

    //attempt to generate new layout
    let mut layout = CompositeLayout::new(&struct_definition.name);
    for member in &struct_definition.members {
      layout.add(
        create_memory_layout(&member.r#type, type_resolver).map_err(|e| {
          MemoryLayoutCreationError {
            target_type_name: target_type_name.to_string(),
            reason: MemoryLayoutCreationErrorReason::MemberLayout {
              member_name: member.name.clone(),
              error: Box::new(e),
            },
          }
        })?,
      )
    }

    let layout: MemoryLayout = layout.into();

    type_resolver
      .struct_definition_cache
      .get_mut(target_type_name)
      .unwrap()
      .1 = Some(layout.clone());

    return Ok(layout);
  }

  Err(MemoryLayoutCreationError {
    target_type_name: target_type_name.to_string(),
    reason: MemoryLayoutCreationErrorReason::UnknownType,
  })
}

#[derive(Debug)]
pub struct MemoryLayoutCreationError {
  pub target_type_name: String,
  pub reason: MemoryLayoutCreationErrorReason,
}

impl Display for MemoryLayoutCreationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "Can't create memory layout for type {}: {}",
      self.target_type_name, self.reason
    )
  }
}

impl Error for MemoryLayoutCreationError {}

#[derive(Debug)]
pub enum MemoryLayoutCreationErrorReason {
  UnknownType,
  MemberLayout {
    member_name: String,
    error: Box<MemoryLayoutCreationError>,
  },
}

impl Display for MemoryLayoutCreationErrorReason {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MemoryLayoutCreationErrorReason::UnknownType => write!(
        f,
        "Unknown type: Not a primitive or previously defined struct"
      ),
      MemoryLayoutCreationErrorReason::MemberLayout { member_name, error } => write!(
        f,
        "Can't create memory layout for member '{}': {}",
        member_name, error
      ),
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MemoryLayout {
  Composed(CompositeLayout),
  Primitive(PrimitiveLayout),
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrimitiveLayout {
  pub primitive_type: String,
}

impl MemoryLayoutCommon for PrimitiveLayout {
  fn iter(&self) -> impl Iterator<Item = &PrimitiveLayout> {
    once(self)
  }
}

pub trait MemoryLayoutCommon {
  fn iter(&self) -> impl Iterator<Item = &PrimitiveLayout>;
}

#[cfg(test)]
mod test {
  use std::collections::{HashMap, HashSet};

  use crate::memory_layout::{
    CompositeLayout, create_memory_layout, MemoryLayout, PrimitiveLayout, TypeResolver,
  };
  use crate::struct_definition::{StructDefinition, StructMember};

  #[test]
  fn test_create_memory_layout() {
    let mut struct_definition_cache: HashMap<String, (StructDefinition, Option<MemoryLayout>)> =
      HashMap::new();
    struct_definition_cache.insert(
      "Pixel".to_string(),
      (
        StructDefinition {
          name: "Pixel".to_string(),
          members: vec![
            StructMember {
              annotation_values: vec![],
              name: "x".to_string(),
              r#type: "f32".to_string(),
            },
            StructMember {
              annotation_values: vec![],
              name: "y".to_string(),
              r#type: "f32".to_string(),
            },
          ],
        },
        None,
      ),
    );
    let mut primitive_types = HashSet::new();
    primitive_types.insert("f32".to_string());
    primitive_types.insert("u32".to_string());
    let mut type_resolver = TypeResolver {
      primitive_types: &primitive_types,
      struct_definition_cache: &mut struct_definition_cache,
    };

    let layout =
      create_memory_layout("Pixel", &mut type_resolver).expect("failed to create memory layout");
    assert_eq!(
      MemoryLayout::Composed(CompositeLayout {
        type_name: "Pixel".to_string(),
        components: vec![
          MemoryLayout::Primitive(PrimitiveLayout {
            primitive_type: "f32".to_string(),
          }),
          MemoryLayout::Primitive(PrimitiveLayout {
            primitive_type: "f32".to_string(),
          }),
        ],
      }),
      layout
    );

    assert_eq!(
      *struct_definition_cache
        .get("Pixel")
        .unwrap()
        .1
        .as_ref()
        .expect("cached memory layout missing"),
      layout
    );
  }
}
