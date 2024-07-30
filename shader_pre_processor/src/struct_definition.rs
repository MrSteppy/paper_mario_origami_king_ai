use std::str::FromStr;

use once_cell_regex::regex;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructDefinition {
  pub name: String,
  pub members: Vec<StructMember>,
}

impl FromStr for StructDefinition {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let struct_regex = regex!(r"^struct (?<name>\S+)\s*\{(?<content>[\s\S]*?)};?");
    let member_regex = regex!(r"\s*(?<annotations>(@\S+\s*)*)(?<name>\S+): (?<type>\S+),\s*");

    let captures = struct_regex
      .captures(s)
      .filter(|caps| caps.get(0).unwrap().start() == 0)
      .ok_or("provided string doesn't start with a valid struct definition")?;
    let struct_name = captures.name("name").unwrap().as_str().to_string();
    let struct_content = captures.name("content").unwrap().as_str();

    let mut members = vec![];
    for captures in member_regex.captures_iter(struct_content) {
      let member_name = captures.name("name").unwrap().as_str().to_string();
      let annotations: Vec<String> = captures
        .name("annotations")
        .unwrap()
        .as_str()
        .split_whitespace()
        .map(|annotation| annotation.to_string())
        .collect();
      let member_type = captures.name("type").unwrap().as_str().to_string();
      members.push(StructMember {
        annotations,
        name: member_name,
        r#type: member_type,
      });
    }

    Ok(StructDefinition {
      name: struct_name,
      members,
    })
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StructMember {
  pub annotations: Vec<String>,
  pub name: String,
  pub r#type: String,
}

#[cfg(test)]
mod test {
  use crate::{pre_process_shader, PreProcessingCache, ProcessContext};
  use crate::struct_definition::StructDefinition;

  #[test]
  fn test_parse_struct_definition() {
    let definition: StructDefinition = r"struct Pixel {
      @location(0) x: u32,
      @location(1) y: u32,
    }"
    .parse()
    .expect("failed to parse struct definition");
    assert_eq!("Pixel", definition.name);
    assert_eq!("x", definition.members[0].name);
    assert_eq!("u32", definition.members[0].r#type);
  }

  #[test]
  fn test_pre_processing() {
    pre_process_shader(
      env!("CARGO_MANIFEST_DIR").to_string() + "/../gui/resources/shader/texture_shader.wgsl",
      ProcessContext::Standalone,
      &mut PreProcessingCache::default(),
      &["f32", "u32"].map(|s| s.to_string()).into_iter().collect()
    )
    .expect("failed to pre-process valid shader code");
  }
}