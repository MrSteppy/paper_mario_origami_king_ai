use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{fs, io};

use lazy_static::lazy_static;
use regex::Regex;
use wgsl_to_wgpu::{create_shader_module, MatrixVectorTypes, WriteOptions};

const INCLUDE_HOOK_POINT: &str = "INCLUDE_HOOK_POINT";
///The prefix of every pre-processor statement
const STMT_PREFIX: &str = "#";
const NO_STANDALONE: &str = "no-standalone";
const INCLUDE: &str = "include";
const INCLUDE_ONLY_ONCE: &str = "once";
const GEN_REPR: &str = "genRepr";

fn main() {
  println!("cargo::rerun-if-changed=resources/shader/**");

  let shader_directory = Path::new("resources/shader");
  let mut shader_rs_source = String::new();

  for entry in fs::read_dir(shader_directory)
    .expect("failed to open shader directory")
    .collect::<Result<Vec<_>, _>>()
    .expect("failed to access file in shader directory")
  {
    let path = entry.path();
    let file_name = entry
      .file_name()
      .to_str()
      .expect("invalid name for shader file")
      .to_string();

    if let Some(shader_name) = file_name.strip_suffix(".wgsl").map(|s| s.to_string()) {
      println!("Processing shader {}...", shader_name);

      let pre_process_info = pre_process_shader(
        path.to_string_lossy().as_ref(),
        None,
        ProcessContext::Standalone,
      )
      .expect("failed to pre-process shader");
      if pre_process_info.no_standalone {
        println!("No standalone - ignoring");
        continue;
      }

      let shader_module_source = create_shader_module(
        &pre_process_info.source_code,
        INCLUDE_HOOK_POINT,
        WriteOptions {
          derive_bytemuck_vertex: true,
          derive_encase_host_shareable: true,
          matrix_vector_types: MatrixVectorTypes::Glam,
          ..Default::default()
        },
      )
      .expect("can't convert shader to rust");
      let shader_module_source = shader_module_source.replace(
        &format!("include_str!(\"{}\")", INCLUDE_HOOK_POINT),
        &format!("r#\"\n{}\"#", pre_process_info.source_code),
      );

      shader_rs_source += &format!("pub mod {} {{\n{}\n}}\n", shader_name, shader_module_source);

      println!("Ok!");
    }
  }

  let shader_rs_path = Path::new("src").join("shader.rs");
  fs::write(&shader_rs_path, shader_rs_source).expect("failed to create shader.rs");
  //try running rust fmt on the file
  if let Ok(mut process) = Command::new("rustfmt").arg(shader_rs_path).spawn() {
    let _ = process.wait();
  }
}

fn pre_process_shader<'a, I>(
  shader_file: &str,
  already_included_files: I,
  context: ProcessContext,
) -> Result<PreProcessingInfo, PreProcessingError>
where
  I: Into<Option<&'a HashSet<String>>>,
{
  let shader_source = fs::read_to_string(shader_file).map_err(|e| PreProcessingError::IO {
    error: e,
    file: Path::new(shader_file).to_path_buf(),
  })?;

  let mut info = PreProcessingInfo {
    included_files: already_included_files.into().cloned().unwrap_or_default(),
    ..Default::default()
  };
  for (line_index, line) in shader_source.lines().enumerate() {
    let line_nr = line_index + 1;
    if get_statement(line, NO_STANDALONE).is_some() {
      info.no_standalone = true;
      if context == ProcessContext::Standalone {
        return Ok(info);
      }
      continue;
    }

    if get_statement(line, INCLUDE_ONLY_ONCE).is_some() {
      info.include_only_once = true;
      if info.included_files.contains(shader_file) {
        return Ok(info);
      }
      continue;
    }

    if let Some(include_statement) = get_statement(line, INCLUDE) {
      let include = &include_statement.arg_str;
      let include_file = Path::new(shader_file)
        .parent()
        .unwrap()
        .join(include)
        .to_string_lossy()
        .to_string();
      let mut include_file_info =
        pre_process_shader(&include_file, &info.included_files, ProcessContext::Include)?;
      if include_file_info.include_only_once && info.included_files.contains(include) {
        continue;
      }
      info.source_code += &include_file_info.source_code;
      info.included_files.insert(include.to_string());
      info
        .struct_definitions
        .append(&mut include_file_info.struct_definitions);
      info
        .generated_representations
        .append(&mut include_file_info.generated_representations);
      for included_file in include_file_info.included_files {
        info.included_files.insert(included_file);
      }
      continue;
    }

    if let Some(gen_repr_statement) = get_statement(line, GEN_REPR) {
      //TODO generate struct representation
      continue;
    }

    //we assume a somewhat formatted shader file
    if let Some(struct_name) = line.strip_prefix("struct ") {
      //TODO parse struct definition
    }

    info.source_code += &format!("{}\n", line);
  }

  Ok(info)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
enum ProcessContext {
  #[default]
  Standalone,
  Include,
}

#[derive(Debug)]
struct StructDefinition {
  name: String,
  members: Vec<StructMember>,
}

#[derive(Debug)]
struct StructMember {
  annotations: Vec<String>,
  name: String,
  r#type: String,
}

impl FromStr for StructDefinition {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    lazy_static! {
      static ref STRUCT_REGEX: Regex =
        Regex::new(r"^struct (\S+) \{([\s\S]*?)};?").expect("invalid regex");
      static ref MEMBER_REGEX: Regex =
        Regex::new(r"\s*((@\S+\s*)*)(\S+): (\S+),\s*").expect("invalid regex");
    }

    todo!()
  }
}

#[derive(Debug)]
struct GeneratedRepresentation {
  name: String,
  ///number of bytes divided by four
  size: usize,
}

fn get_statement(line: &str, statement: &str) -> Option<PreProcessorStatement> {
  let prefix = format!("{}{}", STMT_PREFIX, statement);
  line
    .strip_prefix(&prefix)
    .map(|arg_str| PreProcessorStatement {
      arg_str: arg_str.trim().to_string(),
    })
}

#[derive(Debug)]
struct PreProcessorStatement {
  arg_str: String,
}

#[derive(Debug, Default)]
struct PreProcessingInfo {
  pub included_files: HashSet<String>,
  pub source_code: String,
  pub struct_definitions: Vec<StructDefinition>,
  pub generated_representations: Vec<GeneratedRepresentation>,
  pub no_standalone: bool,
  pub include_only_once: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
enum PreProcessingError {
  IO { error: io::Error, file: PathBuf },
}
