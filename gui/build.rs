use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{fs, io};

use wgsl_to_wgpu::{create_shader_module, MatrixVectorTypes, WriteOptions};

const INCLUDE_HOOK_POINT: &str = "INCLUDE_HOOK_POINT";
const NO_STANDALONE: &str = "//no-standalone";
const INCLUDE: &str = "//include ";
const INCLUDE_ONLY_ONCE: &str = "//once";

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

      let pre_process_info = pre_process_shader(path, None).expect("failed to pre-process shader");
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

fn pre_process_shader<'a, P, I>(
  shader_file: P,
  included_files: I,
) -> Result<PreProcessingInfo, PreProcessingError>
where
  P: AsRef<Path>,
  I: Into<Option<&'a HashSet<String>>>,
{
  let shader_source = fs::read_to_string(&shader_file).map_err(|e| PreProcessingError::IO {
    error: e,
    file: shader_file.as_ref().to_path_buf(),
  })?;

  let mut info = PreProcessingInfo {
    included_files: included_files.into().cloned().unwrap_or_default(),
    ..Default::default()
  };
  for line in shader_source.lines().map(|line| line.trim_end()) {
    if line == NO_STANDALONE {
      info.no_standalone = true;
      continue;
    }

    if line == INCLUDE_ONLY_ONCE {
      info.include_only_once = true;
      continue;
    }

    if let Some(include) = line.strip_prefix(INCLUDE) {
      let include_file = shader_file.as_ref().parent().unwrap().join(include);
      let include_file_info = pre_process_shader(include_file, &info.included_files)?;
      if include_file_info.include_only_once && info.included_files.contains(include) {
        continue;
      }
      info.source_code += &include_file_info.source_code;
      info.included_files.insert(include.to_string());
      for included_file in include_file_info.included_files {
        info.included_files.insert(included_file);
      }
      continue;
    }

    info.source_code += &format!("{}\n", line);
  }

  Ok(info)
}

#[derive(Debug, Default)]
struct PreProcessingInfo {
  pub included_files: HashSet<String>,
  pub source_code: String,
  pub no_standalone: bool,
  pub include_only_once: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
enum PreProcessingError {
  IO { error: io::Error, file: PathBuf },
}
