use std::fmt::{Display, Formatter};
use std::iter::once;

use composite_type::CompositeType;
use primitive_type::PrimitiveType;

use crate::environment::PreProcessingEnvironment;
use crate::memory_layout::{MemoryLayout, PrimitiveMember};
use crate::pre_processing_cache::PreProcessingCache;
use crate::struct_definition::StructDefinition;
use crate::struct_layout::StructLayout;

pub mod composite_type;
pub mod primitive_type;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PrimitiveComposition {
  Primitive(PrimitiveType),
  Composite(CompositeType),
}

impl From<PrimitiveType> for PrimitiveComposition {
  fn from(value: PrimitiveType) -> Self {
    Self::Primitive(value)
  }
}

impl From<CompositeType> for PrimitiveComposition {
  fn from(value: CompositeType) -> Self {
    Self::Composite(value)
  }
}

impl PrimitiveComposition {
  pub fn from_struct_definition<T>(struct_definition: &StructDefinition, resolver: T) -> Self
  where
    T: TypeNameResolver,
  {
    todo!()
  }

  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    match self {
      PrimitiveComposition::Primitive(primitive) => {
        Box::new(once(primitive)) as Box<dyn Iterator<Item = &PrimitiveType>>
      }
      PrimitiveComposition::Composite(composite) => Box::new(composite.primitive_iter()),
    }
  }

  pub fn create_memory_layout(&self) -> MemoryLayout {
    let mut primitive_members: Vec<_> = self
      .primitive_iter()
      .enumerate()
      .map(|(index, primitive)| {
        PrimitiveMember::new(
          PrimitiveMember::member_name_for_index(index),
          primitive.clone(),
        )
      })
      .collect();
    primitive_members.sort_by_cached_key(|member| -(member.r#type.alignment_power as isize));

    let mut number_of_padding_bytes = 0;
    if let Some(alignment_power) = primitive_members
      .first()
      .map(|member| member.r#type.alignment_power)
      .filter(|&power| power > 0)
    {
      let alignment: usize = 1 << alignment_power;
      let layout_size: usize = primitive_members
        .iter()
        .map(|member| member.r#type.size())
        .sum();

      let trailing_bytes = layout_size % alignment;
      if trailing_bytes > 0 {
        number_of_padding_bytes = alignment - trailing_bytes;
      }
    }

    MemoryLayout {
      primitive_members,
      number_of_padding_bytes,
    }
  }

  pub fn name(&self) -> &str {
    match self {
      PrimitiveComposition::Primitive(primitive) => &primitive.name,
      PrimitiveComposition::Composite(composite) => &composite.name,
    }
  }
}

impl Display for PrimitiveComposition {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PrimitiveComposition::Primitive(primitive) => Display::fmt(primitive, f),
      PrimitiveComposition::Composite(composite) => Display::fmt(composite, f),
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub enum TypeRef<'a> {
  StructLayout(&'a StructLayout),
  PrimitiveComposition(&'a PrimitiveComposition),
}

impl<'a> From<&'a StructLayout> for TypeRef<'a> {
  fn from(value: &'a StructLayout) -> Self {
    Self::StructLayout(value)
  }
}

impl<'a> From<&'a PrimitiveComposition> for TypeRef<'a> {
  fn from(value: &'a PrimitiveComposition) -> Self {
    Self::PrimitiveComposition(value)
  }
}

pub trait TypeNameResolver {
  fn resolve(&self, name: &str) -> Option<TypeRef>;
}

#[derive(Debug, Copy, Clone)]
pub struct SimpleStructNameResolver<'a> {
  pub environment: &'a PreProcessingEnvironment,
  pub cache: &'a PreProcessingCache,
}

impl TypeNameResolver for SimpleStructNameResolver<'_> {
  fn resolve(&self, struct_name: &str) -> Option<TypeRef> {
    self
      .environment
      .types()
      .get(struct_name)
      .map(|composition| composition.into())
      .or_else(|| {
        self
          .cache
          .structs()
          .get(struct_name)
          .map(|declaration| (&declaration.declared).into())
      })
  }
}
