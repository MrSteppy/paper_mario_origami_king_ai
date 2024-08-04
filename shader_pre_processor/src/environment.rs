use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::iter::once;
use std::ops::{Add, AddAssign};
use std::path::{Path, PathBuf};

use crate::struct_definition::StructDefinition;
use crate::write_member;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PreProcessingCache {
  pub includes: HashSet<PathBuf>,
  struct_layouts: HashMap<String, Declaration<StructLayout>>,
}

impl PreProcessingCache {
  pub fn structs(&self) -> &HashMap<String, Declaration<StructLayout>> {
    &self.struct_layouts
  }

  pub fn structs_mut(&mut self) -> &mut HashMap<String, Declaration<StructLayout>> {
    &mut self.struct_layouts
  }

  ///inserts a [`Declaration`] in the cache and returns the previous [`Declaration`], if present
  pub fn insert<S>(&mut self, declaration: Declaration<S>) -> Option<Declaration<StructLayout>>
  where
    S: Into<StructLayout>,
  {
    let declaration = declaration.convert(|info, s| info + s.into());
    self
      .struct_layouts
      .insert(declaration.declared.name().to_string(), declaration)
  }

  pub fn update<S>(
    &mut self,
    layout: S,
  ) -> Result<&mut Declaration<StructLayout>, MissingDeclarationError>
  where
    S: Into<StructLayout>,
  {
    let layout = layout.into();
    let declaration = self
      .struct_layouts
      .get_mut(layout.name())
      .ok_or(MissingDeclarationError)?;
    declaration.declared = layout;
    Ok(declaration)
  }
}

#[derive(Debug)]
pub struct MissingDeclarationError;

impl Display for MissingDeclarationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "LayoutDeclaration has not been inserted yet")
  }
}

impl Error for MissingDeclarationError {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Declaration<T> {
  pub info: DeclarationInfo,
  pub declared: T,
}

impl<T> Declaration<T> {
  pub fn new<I, D>(info: I, declared: D) -> Self
  where
    I: Into<DeclarationInfo>,
    D: Into<T>,
  {
    Self {
      info: info.into(),
      declared: declared.into(),
    }
  }

  pub fn separate(self) -> (DeclarationInfo, T) {
    (self.info, self.declared)
  }

  pub fn convert<F, R>(self, mapping: F) -> R
  where
    F: FnOnce(DeclarationInfo, T) -> R,
  {
    mapping(self.info, self.declared)
  }
}

impl<T> Display for Declaration<T>
where
  T: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.info, self.declared)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceLocation {
  pub source_file: PathBuf,
  pub line_nr: usize,
}

impl From<&Path> for SourceLocation {
  fn from(value: &Path) -> Self {
    Self::new(value)
  }
}

impl SourceLocation {
  pub fn new<P>(source_file: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      source_file: source_file.as_ref().to_path_buf(),
      line_nr: 0,
    }
  }

  pub fn at<P>(source_file: P, line_nr: usize) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      source_file: source_file.as_ref().to_path_buf(),
      line_nr,
    }
  }
}

impl Add<usize> for SourceLocation {
  type Output = Self;

  fn add(mut self, rhs: usize) -> Self::Output {
    self.line_nr += rhs;
    self
  }
}

impl AddAssign<usize> for SourceLocation {
  fn add_assign(&mut self, rhs: usize) {
    self.line_nr += rhs;
  }
}

impl Display for SourceLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}:{}", self.source_file, self.line_nr)
  }
}

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationInfo {
  pub source_location: SourceLocation,
}

impl DeclarationInfo {
  pub fn new<S>(source_location: S) -> Self where S: Into<SourceLocation> {
    Self { source_location: source_location.into() }
  }

  pub fn with<T>(self, declared: T) -> Declaration<T> {
    Declaration::new(self.clone(), declared)
  }
}

impl<T> Add<T> for DeclarationInfo {
  type Output = Declaration<T>;

  fn add(self, rhs: T) -> Self::Output {
    self.with(rhs)
  }
}

impl Display for DeclarationInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "at {}", self.source_location)
  }
}

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

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PreProcessingEnvironment {
  primitive_and_native_types: HashMap<String, PrimitiveComposition>,
}

///A native type is a type which is native in wgsl but can not be translated by wgsl_to_wgpu, like mat4x4<f32>.
/// Every type added which is not a [`PrimitiveType`] will be considered native.  
impl PreProcessingEnvironment {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with<T>(mut self, r#type: T) -> Self
  where
    T: Into<PrimitiveComposition>,
  {
    self.add(r#type);
    self
  }

  pub fn add<T>(&mut self, r#type: T)
  where
    T: Into<PrimitiveComposition>,
  {
    let r#type = r#type.into();
    self.primitive_and_native_types.insert(
      match &r#type {
        PrimitiveComposition::Primitive(primitive) => &primitive.name,
        PrimitiveComposition::Composite(native) => &native.name,
      }
      .to_owned(),
      r#type,
    );
  }

  pub fn types(&self) -> &HashMap<String, PrimitiveComposition> {
    &self.primitive_and_native_types
  }
}

///Describes a type which can be directly converted from wgsl to wgpu, like `f32` or `vec4<f32>`.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrimitiveType {
  pub name: String,
  ///the size of this in terms of alignments.
  /// To get the actual size use method [`Self::size`]
  pub size_in_alignments: usize,
  ///the power of two to how many bytes this is aligned
  pub alignment_power: usize,
  ///The fully qualified path to a rust equivalent type
  pub rust_equivalent: String,
}

impl PrimitiveType {
  pub fn new<S, E>(name: S, size: usize, rust_equivalent: E) -> Self
  where
    S: ToString,
    E: ToString,
  {
    Self {
      name: name.to_string(),
      size_in_alignments: size,
      alignment_power: 0,
      rust_equivalent: rust_equivalent.to_string(),
    }
  }

  pub fn new_aligned<S, E>(
    name: S,
    size: usize,
    alignment: usize,
    rust_equivalent: E,
  ) -> Result<Self, PrimitiveTypeCreationError>
  where
    S: ToString,
    E: ToString,
  {
    if !alignment.is_power_of_two() {
      return Err(PrimitiveTypeCreationError::InvalidAlignment);
    }
    if size % alignment != 0 {
      return Err(PrimitiveTypeCreationError::InvalidSize);
    }

    Ok(Self {
      name: name.to_string(),
      size_in_alignments: size / alignment,
      alignment_power: alignment.ilog2() as usize,
      rust_equivalent: rust_equivalent.to_string(),
    })
  }

  #[inline]
  pub const fn size(&self) -> usize {
    (1 << self.alignment_power) * self.size_in_alignments
  }
}

impl PartialOrd for PrimitiveType {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for PrimitiveType {
  fn cmp(&self, other: &Self) -> Ordering {
    self.size().cmp(&other.size())
  }
}

impl Display for PrimitiveType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}[{}{}byte]",
      self.name,
      self.size_in_alignments,
      if self.alignment_power > 0 {
        format!("x{}", self.alignment_power)
      } else {
        "".to_string()
      }
    )
  }
}

#[derive(Debug)]
pub enum PrimitiveTypeCreationError {
  InvalidAlignment,
  InvalidSize,
}

impl Display for PrimitiveTypeCreationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PrimitiveTypeCreationError::InvalidAlignment => "Alignment has to be a power of 2",
        PrimitiveTypeCreationError::InvalidSize => "Size has to be a multiple of alignment",
      }
    )
  }
}

impl Error for PrimitiveTypeCreationError {}

///A type which is composed of multiple other types
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CompositeType {
  pub name: String,
  pub members: Vec<Member>,
}

impl CompositeType {
  pub fn new<S>(name: S) -> Self
  where
    S: ToString,
  {
    Self {
      name: name.to_string(),
      members: vec![],
    }
  }

  pub fn with_member<M>(mut self, member: M) -> Self
  where
    M: Into<Member>,
  {
    self.members.push(member.into());
    self
  }

  pub fn add<M>(&mut self, member: M) -> &mut Self
  where
    M: Into<Member>,
  {
    self.members.push(member.into());
    self
  }

  pub fn primitive_iter(&self) -> impl Iterator<Item = &PrimitiveType> {
    self
      .members
      .iter()
      .flat_map(|member| member.r#type.primitive_iter())
  }
}

impl Display for CompositeType {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} ({})",
      self.name,
      self
        .members
        .iter()
        .map(|member| member.to_string())
        .collect::<Vec<_>>()
        .join(", ")
    )
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Member {
  pub name: String,
  pub r#type: PrimitiveComposition,
  ///The values of all annotations present, without the leading `@`
  pub annotation_values: Vec<String>,
}

impl Member {
  pub fn new<S, T>(name: S, r#type: T) -> Self
  where
    S: ToString,
    T: Into<PrimitiveComposition>,
  {
    Self {
      name: name.to_string(),
      r#type: r#type.into(),
      annotation_values: vec![],
    }
  }

  pub fn new_annotated<A, S, T>(annotations: &[A], name: S, r#type: T) -> Self
  where
    A: ToString,
    S: ToString,
    T: Into<PrimitiveComposition>,
  {
    Self {
      name: name.to_string(),
      r#type: r#type.into(),
      annotation_values: annotations.iter().map(|v| v.to_string()).collect(),
    }
  }
}

impl Display for Member {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write_member(f, &self.annotation_values, &self.name, &self.r#type)
  }
}

///Describes a field in a [`MemoryLayout`]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PrimitiveMember {
  name: String,
  r#type: PrimitiveType,
}

impl PrimitiveMember {
  pub fn member_name_for_index(index: usize) -> String {
    format!("_{index}")
  }

  pub fn new<S, P>(name: S, r#type: P) -> Self
  where
    S: ToString,
    P: Into<PrimitiveType>,
  {
    Self {
      name: name.to_string(),
      r#type: r#type.into(),
    }
  }
}

impl Display for PrimitiveMember {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.name, self.r#type)
  }
}

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

///Describes how a [`PrimitiveComposition`] will be lied out in memory
pub struct MemoryLayout {
  pub primitive_members: Vec<PrimitiveMember>,
  pub number_of_padding_bytes: usize,
}

impl Display for MemoryLayout {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "[{}]",
      self
        .primitive_members
        .iter()
        .map(|member| member.to_string())
        .chain(
          Some(self.number_of_padding_bytes)
            .iter()
            .filter(|&&b| b > 0)
            .map(|b| format!("+{b} padding bytes"))
        )
        .collect::<Vec<_>>()
        .join(", ")
    )
  }
}

#[cfg(test)]
mod test_primitive_type {
  use crate::environment::PrimitiveType;

  #[test]
  fn test_size() {
    assert_eq!(4, PrimitiveType::new("f32", 4, "f32").size());
    assert_eq!(
      16,
      PrimitiveType::new_aligned("vec4<f32>", 16, 16, "glam::Vec4")
        .unwrap()
        .size()
    );
    assert_eq!(
      16,
      PrimitiveType {
        name: "vec2<f64>".to_string(),
        size_in_alignments: 2,
        alignment_power: 3,
        rust_equivalent: "glam::DVec2".to_string(),
      }
      .size()
    );
  }
}

#[cfg(test)]
mod test_composite_type {
  use crate::environment::{CompositeType, Member, PrimitiveType};

  #[test]
  fn test_iter() {
    let number_type = PrimitiveType::new("f32", 4, "f32");
    let pixel_type = CompositeType::new("Pixel")
      .with_member(Member::new("x", number_type.clone()))
      .with_member(Member::new("y", number_type.clone()));
    let str_type = PrimitiveType::new("str", 64, "String");
    let named_pixel_type = CompositeType::new("NamedPixel")
      .with_member(Member::new("name", str_type.clone()))
      .with_member(Member::new("pixel", pixel_type));

    let mut iter = named_pixel_type.primitive_iter();
    assert_eq!(Some(&str_type), iter.next());
    assert_eq!(Some(&number_type), iter.next());
    assert_eq!(Some(&number_type), iter.next());
    assert_eq!(None, iter.next());
  }
}

#[cfg(test)]
mod test_memory_layout_creation {
  use crate::environment::{
    CompositeType, Member, PrimitiveComposition, PrimitiveMember, PrimitiveType,
  };

  #[test]
  fn test_create_memory_layout() {
    let vec4_type = PrimitiveType::new_aligned("vec4<f32>", 16, 16, "glam::Vec4").unwrap();
    let vec3_type = PrimitiveType::new("vec3<f32>", 12, "glam::Vec3");

    let composition = PrimitiveComposition::from(
      CompositeType::new("Vertex")
        .with_member(Member::new("position", vec3_type.clone()))
        .with_member(Member::new("color", vec4_type.clone())),
    );
    let layout = composition.create_memory_layout();
    assert_eq!(
      PrimitiveMember::new("_1", vec4_type.clone()),
      layout.primitive_members[0]
    );
    assert_eq!(
      PrimitiveMember::new("_0", vec3_type.clone()),
      layout.primitive_members[1]
    );
    assert_eq!(4, layout.number_of_padding_bytes);
  }
}
