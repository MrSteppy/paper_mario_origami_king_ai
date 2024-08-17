use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::environment::PreProcessingEnvironment;
use crate::primitive_composition::SimpleStructNameResolver;
use crate::struct_definition::StructDefinition;
use crate::type_analysis::named_type::NamedType;
use enum_assoc::Assoc;
use once_cell_regex::exports::regex::Regex;
use once_cell_regex::regex;
use pre_processing_cache::PreProcessingCache;
use primitive_composition::PrimitiveComposition;
use struct_layout::StructLayout;
use type_analysis::source_location::Declaration;
use type_analysis::TypeDefinitionParseError;

pub mod environment;
pub mod memory_layout;
pub mod pre_processing_cache;
pub mod primitive_composition;
pub mod struct_definition;
pub mod struct_layout;
pub mod type_analysis;

///The prefix of every pre-processor statement
pub const STMT_PREFIX: &str = "#";

///All supported pre-processor statements
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Assoc)]
#[func(pub fn as_str(&self) -> &str)]
#[func(pub fn regex(&self) -> &Regex)]
pub enum Statement {
  ///Mark a file to be ignored by normal shader processing
  #[assoc(as_str = "no-standalone")]
  #[assoc(regex = regex!(r"(?m)^#no-standalone\s*$"))]
  NoStandalone,
  ///Include another file -> Copy-paste it into the code, unless it specifies
  /// otherwise. Parameter must be an absolute or relative path denoting a file.
  #[assoc(as_str = "include")]
  #[assoc(regex = regex!(r"(?m)^#include (?<path>.*?)\s*$"))]
  Include,
  ///If this file gets included, do so only once
  #[assoc(as_str = "once")]
  #[assoc(regex = regex!(r"(?m)^#once\s*$"))]
  IncludeOnlyOnce,
  ///Must annotate a struct - indicates the rust equivalent of a type.
  #[assoc(as_str = "rust")]
  #[assoc(regex = regex!(r"(?m)^#rust (?<equivalent>\S+)\s*$"))]
  Rust,
  ///Must annotate a struct. Generates a serializable representation of the struct, optionally
  /// custom named. Default name is the structs name with a Repr Suffix, ergo for Foo a struct
  /// FooRepr would be generated
  #[assoc(as_str = "data")]
  #[assoc(regex = regex!(r"(?m)^#data (?<name>\S+)?\s*$"))]
  Data,
}

impl Statement {
  pub fn match_line(&self, line: &str) -> Option<StatementInfo> {
    line
      .strip_prefix(&format!("{}{}", STMT_PREFIX, self.as_str()))
      .map(|arg_str| StatementInfo {
        arg_str: arg_str.trim().to_string(),
      })
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StatementInfo {
  pub arg_str: String,
}

///Pre-processes a shader file. Will return None when pre-processing is cancelled early because file
/// has already been included or should not be processed as standalone.
pub fn pre_process_shader<P, C>(
  shader_file: P,
  context: C,
  pre_processing_cache: &mut PreProcessingCache,
  environment: &PreProcessingEnvironment,
) -> Result<Option<String>, PreProcessingError>
where
  P: AsRef<Path>,
  C: Into<ProcessContext>,
{
  let shader_file = shader_file.as_ref();
  let context = context.into();

  let orig_shader_source = fs::read_to_string(shader_file).map_err(|e| PreProcessingError::IO {
    error: e,
    file: shader_file.to_path_buf(),
  })?;

  //TODO first handle imports, after that analyse source code

  let mut source_code = String::new();
  for (line_index, line) in orig_shader_source.lines().enumerate() {
    let line_nr = line_index + 1;

    if Statement::NoStandalone.match_line(line).is_some() {
      if let ProcessContext::Standalone = &context {
        return Ok(None);
      }
      continue;
    }

    if Statement::IncludeOnlyOnce.match_line(line).is_some() {
      if pre_processing_cache.includes.contains(shader_file) {
        return Ok(None);
      }

      continue;
    }

    if let Some(include_info) = Statement::Include.match_line(line) {
      let to_include = &include_info.arg_str;
      let include_path = shader_file
        .parent()
        .expect("can't access shader directory")
        .join(to_include);

      if let Some(include_code) = pre_process_shader(
        include_path,
        ProcessContext::Include,
        pre_processing_cache,
        environment,
      )? {
        source_code += &format!("{include_code}\n");
      }

      continue;
    }

    if let Some(stmt_info) = Statement::Data.match_line(line) {
      //make sure next line has definition
      let mut declaration = pre_processing_cache
        .structs()
        .values()
        .find(|declaration| declaration.info.source_location.line_nr == line_nr + 1)
        .ok_or(PreProcessingError::statement(
          shader_file,
          line_nr,
          line,
          "statement may only annotate a struct",
        ))?
        .clone();

      //parse repr name
      let repr_name = Some(stmt_info.arg_str.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or(format!("{}Repr", declaration.declared.name()));

      //convert layout to primitive composition
      let mut resolver = SimpleStructNameResolver::new(environment, pre_processing_cache);
      //TODO process result

      //TODO create memory layout

      //TODO generate struct representation

      continue;
    }

    source_code += &format!("{line}\n");
  }

  Ok(Some(source_code))
}

fn create_primitive_composition(
  struct_definition: &StructDefinition,
  environment: &PreProcessingEnvironment,
) -> PrimitiveComposition {
  todo!()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ProcessContext {
  #[default]
  Standalone,
  Include,
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
  InvalidStructDefinition(Declaration<TypeDefinitionParseError>),
  StructNameDuplication(Declaration<StructLayout>),
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
      PreProcessingError::InvalidStructDefinition(declaration) => {
        write!(f, "Invalid struct declaration: {declaration}")
      }
      PreProcessingError::StructNameDuplication(previous_declaration) => {
        write!(
          f,
          "A struct with the same name has already been declared {}",
          previous_declaration.info
        )
      }
    }
  }
}

impl Error for PreProcessingError {}

fn write_member<T>(
  f: &mut Formatter,
  annotation_values: &[String],
  name: &str,
  r#type: &T,
) -> std::fmt::Result
where
  T: Display,
{
  write!(
    f,
    "{}{}: {}",
    annotation_values
      .iter()
      .map(|value| format!("@{value} "))
      .collect::<Vec<_>>()
      .join(""),
    name,
    r#type
  )
}

#[cfg(test)]
mod test {
  use crate::environment::PreProcessingEnvironment;
  use crate::pre_processing_cache::PreProcessingCache;
  use crate::{pre_process_shader, ProcessContext};

  #[test]
  fn test_pre_processing() {
    pre_process_shader(
      env!("CARGO_MANIFEST_DIR").to_string() + "/../gui/resources/shader/texture_shader.wgsl",
      ProcessContext::Standalone,
      &mut PreProcessingCache::default(),
      &PreProcessingEnvironment::new(),
    )
    .expect("failed to pre-process valid shader code");
  }
}
