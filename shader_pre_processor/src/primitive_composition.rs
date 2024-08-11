use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::once;

use crate::environment::PreProcessingEnvironment;
use crate::memory_layout::{MemoryLayout, PrimitiveMember};
use crate::pre_processing_cache::PreProcessingCache;
use crate::type_analysis::composite_type::CompositeType;
use crate::type_analysis::declared_type::DeclaredType;
use crate::type_analysis::defined_type::DefinedType;
use crate::type_analysis::member::Member;
use crate::type_analysis::named_type::NamedType;
use crate::type_analysis::primitive_type::PrimitiveType;
use crate::type_analysis::type_declaration::TypeDeclaration;
use crate::type_analysis::TypeNameResolver;

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
    struct_definition: &TypeDeclaration,
    resolver: &mut T,
  ) -> Result<DefinedType, ConversionError>
  where
    T: TypeNameResolver,
  {
    Self::from_struct_definition_with_stack(struct_definition, resolver, &mut vec![])
  }

  pub fn from_struct_definition_with_stack<T>(
    struct_definition: &TypeDeclaration,
    resolver: &mut T,
    processing_stack: &mut Vec<ProcessingStackElement>,
  ) -> Result<DefinedType, ConversionError>
  where
    T: TypeNameResolver,
  {
    let mut composite = CompositeType::new(&struct_definition.name());

    for member in &struct_definition.members {
      let layout = resolver
        .resolve(&member.r#type)
        .ok_or(ConversionError::UnknownType {
          name: member.r#type.clone(),
        })?;
      let member_composition = match layout {
        DeclaredType::Declared(member_type_definition) => {
          processing_stack.push(ProcessingStackElement {
            struct_name: struct_definition.name.clone(),
            field_name: member.name.clone(),
          });
          let type_recursion_stack: Vec<_> = processing_stack
            .iter()
            .skip_while(|element| element.struct_name != member_type_definition.name())
            .cloned()
            .collect();
          if !type_recursion_stack.is_empty() {
            return Err(ConversionError::TypeRecursion {
              processing_stack: type_recursion_stack,
              type_name: member_type_definition.name().to_string(),
            });
          }

          let composition = Self::from_struct_definition_with_stack(
            &member_type_definition,
            resolver,
            processing_stack,
          )?;
          resolver.cache(composition.clone());
          composition
        }
        DeclaredType::Defined(composition) => composition,
      };

      composite.add_member(Member::new_annotated(
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
    let mut primitive_members: Vec<Member<PrimitiveType>> = self
      .primitive_iter()
      .enumerate()
      .map(|(index, primitive)| {
        Member::new(
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

#[derive(Debug)]
pub struct SimpleStructNameResolver<'a> {
  pub environment: &'a PreProcessingEnvironment,
  pub cache: &'a mut PreProcessingCache,
}

impl<'a> SimpleStructNameResolver<'a> {
  pub fn new(environment: &'a PreProcessingEnvironment, cache: &'a mut PreProcessingCache) -> Self {
    Self { environment, cache }
  }
}

impl TypeNameResolver for SimpleStructNameResolver<'_> {
  fn resolve(&self, struct_name: &str) -> Option<DeclaredType> {
    self
      .environment
      .types()
      .get(struct_name)
      .map(|composition| composition.clone().into())
      .or_else(|| {
        self
          .cache
          .structs()
          .get(struct_name)
          .map(|declaration| declaration.declared.clone().into())
      })
  }

  fn cache(&mut self, primitive_composition: DefinedType) {
    self
      .cache
      .update(primitive_composition)
      .expect("caching primitive composition without cached declaration");
  }
}

#[cfg(test)]
mod test {
  use crate::environment::PreProcessingEnvironment;
  use crate::pre_processing_cache::PreProcessingCache;
  use crate::primitive_composition::{PrimitiveComposition, SimpleStructNameResolver};
  use crate::type_analysis::composite_type::CompositeType;
  use crate::type_analysis::defined_type::DefinedType;
  use crate::type_analysis::member::Member;
  use crate::type_analysis::primitive_type::PrimitiveType;
  use crate::type_analysis::type_declaration::TypeDeclaration;

  #[test]
  fn test_from_struct_definition() {
    let struct_definition = TypeDeclaration::new("Pixel")
      .with_member(Member::new("x", "u32"))
      .with_member(Member::new("y", "u32"));
    let u32_type = PrimitiveType::new("u32", 4, "u32");
    let environment = PreProcessingEnvironment::new().with(u32_type.clone());
    let mut cache = PreProcessingCache::new();
    let mut resolver = SimpleStructNameResolver::new(&environment, &mut cache);

    let composition =
      PrimitiveComposition::from_struct_definition(&struct_definition, &mut resolver)
        .expect("conversion error");

    assert_eq!(
      DefinedType::Composite(
        CompositeType::new("Pixel")
          .with_member(Member::new("x", u32_type.clone()))
          .with_member(Member::new("y", u32_type.clone()))
      ),
      composition
    );
  }
}
