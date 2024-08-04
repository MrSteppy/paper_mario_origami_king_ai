use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::once;

use composite_type::CompositeType;
use primitive_type::PrimitiveType;

use crate::environment::PreProcessingEnvironment;
use crate::memory_layout::{MemoryLayout, PrimitiveMember};
use crate::pre_processing_cache::PreProcessingCache;
use crate::primitive_composition::composite_type::Member;
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
  pub fn from_struct_definition<T>(
    struct_definition: &StructDefinition,
    resolver: &T,
  ) -> Result<Self, ConversionError>
  where
    T: TypeNameResolver,
  {
    Self::from_struct_definition_with_stack(struct_definition, resolver, &mut vec![])
  }

  pub fn from_struct_definition_with_stack<T>(
    struct_definition: &StructDefinition,
    resolver: &T,
    processing_stack: &mut Vec<ProcessingStackElement>,
  ) -> Result<Self, ConversionError>
  where
    T: TypeNameResolver,
  {
    let mut composite = CompositeType::new(&struct_definition.name);

    for member in &struct_definition.members {
      let type_ref = resolver
        .resolve(&member.type_name)
        .ok_or(ConversionError::UnknownType {
          name: member.type_name.clone(),
        })?;
      let owned_composition;
      let member_composition = match type_ref {
        TypeRef::StructLayout(layout_ref) => match layout_ref {
          StructLayout::Simple(member_type_definition) => {
            processing_stack.push(ProcessingStackElement {
              struct_name: struct_definition.name.clone(),
              field_name: member.name.clone(),
            });
            let type_recursion_stack: Vec<_> = processing_stack
              .iter()
              .skip_while(|element| element.struct_name != member_type_definition.name)
              .cloned()
              .collect();
            if !type_recursion_stack.is_empty() {
              return Err(ConversionError::TypeRecursion {
                processing_stack: type_recursion_stack,
                type_name: member_type_definition.name.clone(),
              });
            }

            owned_composition = Self::from_struct_definition_with_stack(
              member_type_definition,
              resolver,
              processing_stack,
            )?;
            &owned_composition
          }
          StructLayout::Detailed { composition, .. } => composition,
        },
        TypeRef::PrimitiveComposition(composition) => composition,
      };

      composite.add(Member::new_annotated(
        &member.annotation_values,
        &member.name,
        member_composition.clone(),
      ));
    }

    Ok(composite.into())
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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ProcessingStackElement {
  pub struct_name: String,
  pub field_name: String,
}

impl Display for ProcessingStackElement {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}::{}", self.struct_name, self.field_name)
  }
}

#[derive(Debug)]
pub enum ConversionError {
  UnknownType {
    name: String,
  },
  TypeRecursion {
    processing_stack: Vec<ProcessingStackElement>,
    type_name: String,
  },
}

impl Display for ConversionError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ConversionError::UnknownType { name } => write!(f, "Unknown type: {name}"),
      ConversionError::TypeRecursion {
        processing_stack,
        type_name,
      } => write!(
        f,
        "Struct contains itself: {} -> {type_name}",
        processing_stack
          .iter()
          .map(|element| element.to_string())
          .collect::<Vec<_>>()
          .join(" -> ")
      ),
    }
  }
}

impl Error for ConversionError {}

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

impl<'a> SimpleStructNameResolver<'a> {
  pub fn new(environment: &'a PreProcessingEnvironment, cache: &'a PreProcessingCache) -> Self {
    Self { environment, cache }
  }
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

#[cfg(test)]
mod test {
  use crate::environment::PreProcessingEnvironment;
  use crate::pre_processing_cache::PreProcessingCache;
  use crate::primitive_composition::{PrimitiveComposition, SimpleStructNameResolver};
  use crate::primitive_composition::composite_type::{CompositeType, Member};
  use crate::primitive_composition::primitive_type::PrimitiveType;
  use crate::struct_definition::{StructDefinition, StructMember};

  #[test]
  fn test_from_struct_definition() {
    let struct_definition = StructDefinition::new("Pixel")
      .with(StructMember::new("x", "u32"))
      .with(StructMember::new("y", "u32"));
    let u32_type = PrimitiveType::new("u32", 4, "u32");
    let environment = PreProcessingEnvironment::new().with(u32_type.clone());
    let cache = PreProcessingCache::new();
    let resolver = SimpleStructNameResolver::new(&environment, &cache);

    let composition = PrimitiveComposition::from_struct_definition(&struct_definition, &resolver)
      .expect("conversion error");

    assert_eq!(
      PrimitiveComposition::Composite(
        CompositeType::new("Pixel")
          .with_member(Member::new("x", u32_type.clone()))
          .with_member(Member::new("y", u32_type.clone()))
      ),
      composition
    );
  }
}
