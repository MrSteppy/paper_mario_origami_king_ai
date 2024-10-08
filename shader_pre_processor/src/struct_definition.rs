use std::error::Error;
use std::fmt::{Display, Formatter};

use once_cell_regex::exports::regex::{Captures, Regex};
use once_cell_regex::regex;

use crate::type_analysis::source_location::{Declaration, DeclarationInfo, SourceLocation};
use crate::type_analysis::TypeDefinitionParseError;
use crate::write_member;

///deprecated: use [`crate::type_analysis::declared_type::DeclaredType`] instead
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[deprecated]
pub struct StructDefinition {
  pub name: String,
  pub members: Vec<StructMember>,
}

impl StructDefinition {
  pub fn new<S>(name: S) -> Self
  where
    S: ToString,
  {
    Self {
      name: name.to_string(),
      members: vec![],
    }
  }

  pub fn with<M>(mut self, member: M) -> Self
  where
    M: Into<StructMember>,
  {
    self.members.push(member.into());
    self
  }

  fn struct_regex() -> &'static Regex {
    regex!(r"struct (?<name>\S+)\s*\{(?<content>[\s\S]*?)};?")
  }

  fn member_regex() -> &'static Regex {
    regex!(r"\s*(?<annotations>(@\S+\s*)*)(?<name>\S+): (?<type>\S+),\s*")
  }

  #[deprecated]
  pub fn from_shader_source<S, L>(
    shader_source: S,
    source_location: L,
  ) -> Vec<Declaration<Result<StructDefinition, TypeDefinitionParseError>>>
  where
    S: AsRef<str>,
    L: Into<SourceLocation>,
  {
    let shader_source = shader_source.as_ref();
    let source_location = source_location.into();

    let mut struct_definitions = vec![];
    for struct_captures in Self::struct_regex().captures_iter(shader_source) {
      //substring via byte index since Match::start is in bytes
      let struct_match = struct_captures.get(0).expect("i == 0 => Some");
      let line_nr = shader_source[..struct_match.start()]
        .chars()
        .filter(|&c| c == '\n')
        .count()
        + 1;

      struct_definitions.push(Declaration::new(
        DeclarationInfo::new(source_location.clone() + line_nr),
        Self::from_captures(struct_captures),
      ));
    }
    struct_definitions
  }

  fn from_captures(captures: Captures) -> Result<StructDefinition, TypeDefinitionParseError> {
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
    for captures in Self::member_regex().captures_iter(struct_content) {
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
      members.push(StructMember {
        annotation_values: annotations,
        name: member_name,
        type_name: member_type,
      });
    }

    Ok(StructDefinition { name, members })
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructMember {
  pub annotation_values: Vec<String>,
  pub name: String,
  pub type_name: String,
}

impl StructMember {
  pub fn new<S, T>(name: S, type_name: T) -> Self
  where
    S: ToString,
    T: ToString,
  {
    Self::new_annotated::<&str, _, _>(&[], name, type_name)
  }

  #[inline]
  pub fn new_annotated<A, S, T>(annotation_values: &[A], name: S, type_name: T) -> Self
  where
    A: ToString,
    S: ToString,
    T: ToString,
  {
    Self {
      annotation_values: annotation_values.iter().map(|a| a.to_string()).collect(),
      name: name.to_string(),
      type_name: type_name.to_string(),
    }
  }
}

impl Display for StructMember {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write_member(f, &self.annotation_values, &self.name, &self.type_name)
  }
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use crate::struct_definition::{StructDefinition, StructMember};
  use crate::type_analysis::source_location::{Declaration, DeclarationInfo, SourceLocation};

  #[test]
  fn test_parse_struct_definition() {
    let shader_source = r"struct Pixel {
      @location(0) x: u32,
      @location(1) y: u32,
    }";
    let source_path = Path::new(":memory:");
    let definitions = StructDefinition::from_shader_source(shader_source, source_path);
    assert_eq!(
      vec![Declaration::new(
        DeclarationInfo::new(SourceLocation::at(source_path, 1)),
        Ok(StructDefinition {
          name: "Pixel".to_string(),
          members: vec![
            StructMember {
              annotation_values: vec!["location(0)".to_string()],
              name: "x".to_string(),
              type_name: "u32".to_string(),
            },
            StructMember {
              annotation_values: vec!["location(1)".to_string()],
              name: "y".to_string(),
              type_name: "u32".to_string(),
            }
          ],
        })
      )],
      definitions
    );
  }
}
