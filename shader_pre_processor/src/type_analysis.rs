use crate::type_analysis::declared_type::DeclaredType;
use crate::type_analysis::defined_type::DefinedType;
use crate::type_analysis::member::Member;
use crate::type_analysis::source_location::SourceLocation;
use crate::type_analysis::type_declaration::TypeDeclaration;
use once_cell_regex::exports::regex::Captures;
use once_cell_regex::regex;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub mod composite_type;
pub mod declared_type;
pub mod defined_type;
pub mod member;
pub mod named_type;
pub mod primitive_type;
pub mod source_location;
pub mod type_declaration;

///Extracts all struct declarations from a given shader source. 
/// Only parses native wgsl code, does not parse pre-processor annotations like rust equivalents!
pub fn parse_type_declarations<S, L>(
  shader_source: S,
  source_location: L,
) -> Vec<(
  SourceLocation,
  Result<TypeDeclaration, TypeDefinitionParseError>,
)>
where
  S: AsRef<str>,
  L: Into<SourceLocation>,
{
  let shader_source = shader_source.as_ref();
  let source_location = source_location.into();

  let mut type_declarations = vec![];
  for struct_captures in
    regex!(r"struct (?<name>\S+)\s*\{(?<content>[\s\S]*?)};?").captures_iter(shader_source)
  {
    //substring via byte index since Match::start is in bytes
    let struct_match = struct_captures
      .get(0)
      .expect("zeroth capture group should always exist");
    let line_nr = shader_source[..struct_match.start()]
      .chars()
      .filter(|&c| c == '\n')
      .count()
      + 1;

    type_declarations.push((
      source_location.clone() + line_nr,
      type_declaration_from_captures(struct_captures),
    ));
  }
  type_declarations
}

fn type_declaration_from_captures(
  captures: Captures,
) -> Result<TypeDeclaration, TypeDefinitionParseError> {
  let name = captures
    .name("name")
    .expect("missing capture group")
    .as_str()
    .to_string();
  let struct_content = captures
    .name("content")
    .expect("missing capture group")
    .as_str();

  let mut members = vec![];
  for captures in regex!(r"\s*(?<annotations>(@\S+\s*)*)(?<name>\S+): (?<type>\S+),\s*")
    .captures_iter(struct_content)
  {
    let member_name = captures
      .name("name")
      .expect("missing capture group")
      .as_str()
      .to_string();
    let annotations: Vec<String> = captures
      .name("annotations")
      .expect("missing capture group")
      .as_str()
      .split_whitespace()
      .map(|annotation| {
        annotation
          .strip_prefix('@')
          .map(|annotation_value| annotation_value.to_string())
          .ok_or(TypeDefinitionParseError::MissingAnnotationPrefix {
            member_name: member_name.clone(),
            annotation: annotation.to_string(),
          })
      })
      .collect::<Result<Vec<_>, TypeDefinitionParseError>>()?;
    let member_type = captures
      .name("type")
      .expect("missing capture group")
      .as_str()
      .to_string();
    members.push(Member::new_annotated(
      &annotations,
      member_name,
      member_type,
    ));
  }

  let mut declaration = TypeDeclaration::new(name);
  declaration.members = members;
  Ok(declaration)
}

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeDefinitionParseError {
  MissingAnnotationPrefix {
    member_name: String,
    annotation: String,
  },
}

impl Display for TypeDefinitionParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      TypeDefinitionParseError::MissingAnnotationPrefix {
        member_name,
        annotation,
      } => write!(
        f,
        "annotation on member {member_name} is missing annotation prefix(@): '{annotation}'"
      ),
    }
  }
}

impl Error for TypeDefinitionParseError {}

pub trait TypeNameResolver {
  fn resolve(&self, name: &str) -> Option<DeclaredType>;

  fn cache(&mut self, primitive_composition: DefinedType);
}
