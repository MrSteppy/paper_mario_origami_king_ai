use std::error::Error;
use std::fmt::{Display, Formatter};

use once_cell_regex::exports::regex::{Captures, Regex};
use once_cell_regex::regex;

use crate::declaration::{Declaration, DeclarationInfo, SourceLocation};
use crate::write_member;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructDefinition {
  pub name: String,
  pub members: Vec<StructMember>,
}

impl StructDefinition {
  fn struct_regex() -> &'static Regex {
    regex!(r"struct (?<name>\S+)\s*\{(?<content>[\s\S]*?)};?")
  }

  fn member_regex() -> &'static Regex {
    regex!(r"\s*(?<annotations>(@\S+\s*)*)(?<name>\S+): (?<type>\S+),\s*")
  }

  pub fn from_source<S, L>(
    shader_source: S,
    source_location: L,
  ) -> Vec<Declaration<Result<StructDefinition, StructDefinitionError>>>
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

  fn from_captures(captures: Captures) -> Result<StructDefinition, StructDefinitionError> {
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
            .ok_or(StructDefinitionError::MissingAnnotationPrefix {
              member_name: member_name.clone(),
              annotation: annotation.to_string(),
            })
        })
        .collect::<Result<Vec<_>, StructDefinitionError>>()?;
      let member_type = captures
        .name("type")
        .expect("missing capture group")
        .as_str()
        .to_string();
      members.push(StructMember {
        annotation_values: annotations,
        name: member_name,
        r#type: member_type,
      });
    }

    Ok(StructDefinition { name, members })
  }
}

#[non_exhaustive]
#[derive(Debug, Eq, PartialEq)]
pub enum StructDefinitionError {
  MissingAnnotationPrefix {
    member_name: String,
    annotation: String,
  },
}

impl Display for StructDefinitionError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      StructDefinitionError::MissingAnnotationPrefix {
        member_name,
        annotation,
      } => write!(
        f,
        "annotation on member {member_name} is missing annotation prefix(@): '{annotation}'"
      ),
    }
  }
}

impl Error for StructDefinitionError {}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructMember {
  pub annotation_values: Vec<String>,
  pub name: String,
  pub r#type: String,
}

impl Display for StructMember {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write_member(f, &self.annotation_values, &self.name, &self.r#type)
  }
}

#[cfg(test)]
mod test {
  use std::path::Path;

  use crate::declaration::{Declaration, DeclarationInfo, SourceLocation};
  use crate::struct_definition::{StructDefinition, StructMember};

  #[test]
  fn test_parse_struct_definition() {
    let shader_source = r"struct Pixel {
      @location(0) x: u32,
      @location(1) y: u32,
    }";
    let source_path = Path::new(":memory:");
    let definitions = StructDefinition::from_source(shader_source, source_path);
    assert_eq!(
      vec![Declaration::new(
        DeclarationInfo::new(SourceLocation::at(source_path, 1)),
        Ok(StructDefinition {
          name: "Pixel".to_string(),
          members: vec![
            StructMember {
              annotation_values: vec!["location(0)".to_string()],
              name: "x".to_string(),
              r#type: "u32".to_string(),
            },
            StructMember {
              annotation_values: vec!["location(1)".to_string()],
              name: "y".to_string(),
              r#type: "u32".to_string(),
            }
          ],
        })
      )],
      definitions
    );
  }
}
