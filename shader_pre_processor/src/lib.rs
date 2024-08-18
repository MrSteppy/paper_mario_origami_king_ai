use crate::environment::PreProcessingEnvironment;
use crate::primitive_composition::SimpleStructNameResolver;
use crate::struct_definition::StructDefinition;
use crate::type_analysis::named_type::NamedType;
use crate::type_analysis::source_location::SourceLocation;
use enum_assoc::Assoc;
use pre_processing_cache::PreProcessingCache;
use primitive_composition::PrimitiveComposition;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::{fs, io};
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
pub enum Statement {
  ///Mark a file to be ignored by normal shader processing
  #[assoc(as_str = "no-standalone")]
  NoStandalone,
  ///<path to file> Include another file -> Copy-paste it into the code, unless it specifies
  /// otherwise. Parameter must be an absolute or relative path denoting a file.
  #[assoc(as_str = "include")]
  Include,
  ///If this file gets included, do so only once
  #[assoc(as_str = "once")]
  IncludeOnlyOnce,
  ///<rust_equivalent> - Must annotate a struct - indicates the rust equivalent of a type.
  #[assoc(as_str = "rust")]
  Rust,
  ///\[repr_name] - Must annotate a struct. Generates a serializable representation of the struct, optionally
  /// custom named. Default name is the structs name with a Repr Suffix, ergo for Foo a struct
  /// FooRepr would be generated
  #[assoc(as_str = "data")]
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

  pub fn find_usages<'a>(
    &self,
    shader_source: &'a str,
  ) -> impl Iterator<Item = StatementUsage> + 'a {
    let pattern = format!("{STMT_PREFIX}{}", self.as_str());
    shader_source
      .lines()
      .enumerate()
      .filter_map(move |(line_index, line)| {
        line.trim_start().strip_prefix(&pattern).map(|arg_str| {
          let arg_str = arg_str.trim().to_string();
          let line_nr = unsafe { NonZeroUsize::new_unchecked(line_index + 1) };
          StatementUsage { line_nr, arg_str }
        })
      })
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StatementUsage {
  pub line_nr: NonZeroUsize,
  pub arg_str: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StatementInfo {
  pub arg_str: String,
}

///Pre-processes a shader file. Will return None when pre-processing is cancelled early because file
/// has already been included or should not be processed as standalone.
//TODO fix return type: multiple warns are always possible, additionally either multiple errors or an ok value, which may have source code
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
  let mut line_replacements: HashMap<usize, String> = HashMap::new();
  let no_standalone_usages: Vec<_> = Statement::NoStandalone
    .find_usages(&orig_shader_source)
    .collect();
  if !no_standalone_usages.is_empty() {
    if context == ProcessContext::Standalone {
      return Ok(None);
    }

    for usage in no_standalone_usages {
      line_replacements.insert(usage.line_nr.get(), usage.arg_str);
    }
  }

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

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PreProcessingResult {
  //TODO result -> Result<ProcessedSource, Vec<PreProcessingError>>
  pub warnings: Vec<PreProcessingWarning>
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PreProcessingWarning {
  pub source_location: SourceLocation,
  pub detail_message: String,
}

impl Display for PreProcessingWarning {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "[WARNING] {} (at {})", self.detail_message, self.source_location)
  }
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
  use crate::{pre_process_shader, ProcessContext, Statement, StatementUsage};
  use std::num::NonZeroUsize;

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

  #[test]
  fn test_find_statement_usages() {
    let source = "#include foo\n  #include bar\n//#include var";
    let mut usage_iter = Statement::Include.find_usages(source);
    assert_eq!(
      Some(StatementUsage {
        line_nr: NonZeroUsize::new(1).unwrap(),
        arg_str: "foo".to_string()
      }),
      usage_iter.next()
    );
    assert_eq!(
      Some(StatementUsage {
        line_nr: NonZeroUsize::new(2).unwrap(),
        arg_str: "bar".to_string()
      }),
      usage_iter.next()
    );
    assert_eq!(None, usage_iter.next());
  }
}
