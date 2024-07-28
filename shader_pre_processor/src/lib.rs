use std::{fs, io};
use std::collections::HashSet;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};

use enum_assoc::Assoc;

use struct_definition::StructDefinition;

use crate::memory_layout::{create_memory_layout, StructDefinitionCache, TypeResolver};

mod memory_layout;
mod struct_definition;

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
pub fn pre_process_shader<P, C>(
  shader_file: P,
  context: C,
  pre_processing_cache: &mut PreProcessingCache,
  primitive_types: &HashSet<String>,
) -> Result<Option<String>, PreProcessingError>
where
  P: AsRef<Path>,
  C: Into<ProcessContext>,
{
  let shader_file = shader_file.as_ref();
  let context = context.into();

  let shader_source = fs::read_to_string(shader_file).map_err(|e| PreProcessingError::IO {
    error: e,
    file: shader_file.to_path_buf(),
  })?;

  let mut source_code = String::new();
  for (line_index, line) in shader_source.lines().enumerate() {
    let line_nr = line_index + 1;

    if Statement::NoStandalone.match_info(line).is_some() {
      if let ProcessContext::Standalone = &context {
        return Ok(None);
      }
      continue;
    }

    if Statement::IncludeOnlyOnce.match_info(line).is_some() {
      if pre_processing_cache.includes.contains(shader_file) {
        return Ok(None);
      }

      continue;
    }

    if let Some(include_info) = Statement::Include.match_info(line) {
      let to_include = &include_info.arg_str;
      let include_path = shader_file
        .parent()
        .expect("can't access shader directory")
        .join(to_include);

      if let Some(include_code) = pre_process_shader(
        include_path,
        ProcessContext::Include,
        pre_processing_cache,
        primitive_types,
      )? {
        source_code += &format!("{include_code}\n");
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
      let _repr_name = args
        .next()
        .map(|s| s.to_string())
        .unwrap_or(format!("{}Repr", target_name));

      let _memory_layout = create_memory_layout(
        target_name,
        &mut TypeResolver {
          primitive_types,
          struct_definition_cache: &mut pre_processing_cache.struct_definition_cache,
        },
      )
      .map_err(|e| {
        PreProcessingError::statement(
          shader_file,
          line_nr,
          line,
          format!("Failed to create memory layout: {e}"),
        )
      })?;

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
      pre_processing_cache
        .struct_definition_cache
        .insert(struct_definition.name.clone(), (struct_definition, None));
    }

    source_code += &format!("{line}\n");
  }

  Ok(Some(source_code))
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub enum ProcessContext {
  #[default]
  Standalone,
  Include,
}

#[derive(Debug, Clone, Default)]
pub struct PreProcessingCache {
  pub includes: HashSet<PathBuf>,
  pub struct_definition_cache: StructDefinitionCache,
  pub generated_representations: Vec<GeneratedRepresentation>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GeneratedRepresentation {
  pub name: String,
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
