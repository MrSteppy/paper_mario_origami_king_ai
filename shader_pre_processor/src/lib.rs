use std::{fs, io};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use enum_assoc::Assoc;
use once_cell_regex::regex;

mod memory_layout;

///The prefix of every pre-processor statement
pub const STMT_PREFIX: &str = "#";

///All supported pre-processor statements
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Assoc)]
#[func(pub fn as_str(&self) -> &str)]
pub enum Statement {
  ///Mark a file to be ignored by normal shader processing
  #[assoc(as_str = "no-standalone")]
  NoStandalone,
  ///<file> - Include another file -> Copy-paste it into the code
  #[assoc(as_str = "include")]
  Include,
  ///If this file gets included, do so only once
  #[assoc(as_str = "once")]
  IncludeOnlyOnce,
  ///<struct> [repr-name] - Generate a representation of the struct which can be translated by wgsl_to_wgpu.
  #[assoc(as_str = "genRepr")]
  GenRepr,
}

impl Statement {
  pub fn match_info(&self, line: &str) -> Option<StatementInfo> {
    line
      .strip_prefix(&format!("{}{}", STMT_PREFIX, self.as_str()))
      .map(|arg_str| StatementInfo {
        arg_str: arg_str.trim().to_string(),
      })
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StatementInfo {
  arg_str: String,
}

///Pre-processes a shader file. Will return None when pre-processing is cancelled early because file
/// has already been included or should not be processed as standalone.
pub fn pre_process_shader<'a, P, C>(
  shader_file: P,
  context: C,
) -> Result<Option<PreProcessedShaderInfo>, PreProcessingError>
where
  P: AsRef<Path>,
  C: Into<ProcessContext<'a>>,
{
  let shader_file = shader_file.as_ref();
  let context = context.into();

  let shader_source = fs::read_to_string(shader_file).map_err(|e| PreProcessingError::IO {
    error: e,
    file: shader_file.to_path_buf(),
  })?;

  let mut info = PreProcessedShaderInfo {
    included_files: Default::default(),
    ..Default::default()
  };
  for (line_index, line) in shader_source.lines().enumerate() {
    let line_nr = line_index + 1;
    if Statement::NoStandalone.match_info(line).is_some() {
      info.no_standalone = true;
      if let ProcessContext::Standalone = &context {
        return Ok(None);
      }
      continue;
    }

    if Statement::IncludeOnlyOnce.match_info(line).is_some() {
      info.include_only_once = true;
      let shader_file_str = shader_file
        .to_str()
        .expect("shader file path not valid utf-8, should not have been able to be included")
        .to_string();
      if context
        .join(&info)
        .previous_includes
        .contains(&shader_file_str)
      {
        return Ok(None);
      };
      continue;
    }

    if let Some(stmt_info) = Statement::Include.match_info(line) {
      let to_include = &stmt_info.arg_str;
      let include_file = shader_file
        .parent()
        .expect("can't access shader directory")
        .join(to_include);

      if let Some(include_file_info) = pre_process_shader(include_file, context.join(&info))? {
        info.absorb(include_file_info);
      }

      continue;
    }

    if let Some(stmt_info) = Statement::GenRepr.match_info(line) {
      let mut args = stmt_info.arg_str.split_whitespace();
      let target_name = args.next().ok_or(PreProcessingError::statement(
        shader_file,
        line_nr,
        line,
        "Missing target name: For which struct shall the repr be generated?",
      ))?;
      let repr_name = args
        .next()
        .map(|s| s.to_string())
        .unwrap_or(format!("{}Repr", target_name));

      let target_definition = context
        .join(&info)
        .previous_struct_definitions
        .iter()
        .find(|definition| definition.name == target_name)
        .ok_or(PreProcessingError::statement(shader_file, line_nr, line, format!("Can't find a struct with the name '{}'. Make sure it was defined before this statement!", target_name)))?;



      //TODO generate struct representation
      continue;
    }

    if let Ok(struct_definition) = shader_source
      .lines()
      .skip(line_index)
      .collect::<Vec<_>>()
      .join("\n")
      .parse::<StructDefinition>()
    {
      info.struct_definitions.push(struct_definition);
    }

    info.source_code += &format!("{}\n", line);
  }

  Ok(Some(info))
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct IncludeContext<'a> {
  pub previous_includes: HashSet<&'a String>,
  pub previous_struct_definitions: Vec<&'a StructDefinition>,
}

impl<'a> IncludeContext<'a> {
  pub fn join(&'a self, info: &'a PreProcessedShaderInfo) -> IncludeContext<'a> {
    let mut context = Self::from(info);
    for &include in &self.previous_includes {
      context.previous_includes.insert(include);
    }
    for &struct_def in &self.previous_struct_definitions {
      context.previous_struct_definitions.push(struct_def);
    }

    context
  }
}

impl<'a> From<&'a PreProcessedShaderInfo> for IncludeContext<'a> {
  fn from(value: &'a PreProcessedShaderInfo) -> Self {
    let mut context = Self::default();

    for include in &value.included_files {
      context.previous_includes.insert(include);
    }
    for struct_definition in &value.struct_definitions {
      context.previous_struct_definitions.push(struct_definition);
    }

    context
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub enum ProcessContext<'a> {
  #[default]
  Standalone,
  Include(IncludeContext<'a>),
}

impl<'a> From<IncludeContext<'a>> for ProcessContext<'a> {
  fn from(value: IncludeContext<'a>) -> Self {
    Self::Include(value)
  }
}

impl<'a> ProcessContext<'a> {
  pub fn join(&'a self, info: &'a PreProcessedShaderInfo) -> IncludeContext<'a> {
    match self {
      ProcessContext::Standalone => IncludeContext::from(info),
      ProcessContext::Include(inner) => inner.join(info),
    }
  }
}

#[derive(Debug, Default)]
pub struct PreProcessedShaderInfo {
  pub included_files: HashSet<String>,
  pub source_code: String,
  pub struct_definitions: Vec<StructDefinition>,
  pub generated_representations: Vec<GeneratedRepresentation>,
  pub no_standalone: bool,
  pub include_only_once: bool,
}

impl PreProcessedShaderInfo {
  pub fn absorb(&mut self, info: PreProcessedShaderInfo) {
    let PreProcessedShaderInfo {
      included_files,
      source_code,
      mut struct_definitions,
      mut generated_representations,
      ..
    } = info;
    for include in included_files {
      self.included_files.insert(include);
    }
    self.source_code += &source_code;
    self.struct_definitions.append(&mut struct_definitions);
    self
      .generated_representations
      .append(&mut generated_representations);
  }
}

#[derive(Debug)]
pub struct GeneratedRepresentation {
  pub name: String,
  ///number of bytes divided by four
  pub size: usize,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum PreProcessingError {
  IO {
    error: io::Error,
    file: PathBuf,
  },
  Statement {
    file: PathBuf,
    line_nr: usize,
    line: String,
    detail_message: String,
  },
}

impl PreProcessingError {
  pub fn statement<P, L, S>(file: P, line_nr: usize, line: L, detail_message: S) -> Self
  where
    P: AsRef<Path>,
    L: ToString,
    S: ToString,
  {
    Self::Statement {
      file: file.as_ref().to_path_buf(),
      line_nr,
      line: line.to_string(),
      detail_message: detail_message.to_string(),
    }
  }
}

impl Display for PreProcessingError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      PreProcessingError::IO { error, file } => {
        write!(f, "IO error with file {:?}: {}", file, error)
      }
      PreProcessingError::Statement {
        file,
        line_nr,
        line,
        detail_message,
      } => write!(
        f,
        "Invalid statement at {:?}:{} near '{}': {}",
        file, line_nr, line, detail_message
      ),
    }
  }
}

impl Error for PreProcessingError {}

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
  use crate::{pre_process_shader, ProcessContext, StructDefinition};

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
    )
    .expect("failed to pre-process valid shader code");
  }
}
