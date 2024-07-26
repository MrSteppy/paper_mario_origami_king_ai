use std::{fs, io};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use wgsl_to_wgpu::{create_shader_module, MatrixVectorTypes, WriteOptions};

const INCLUDE_HOOK_POINT: &str = "INCLUDE_HOOK_POINT";
const NO_STANDALONE: &str = "//no-standalone";

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

      let pre_process_info = pre_process_shader(path).expect("failed to pre-process shader");
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

fn pre_process_shader<P>(shader_file: P) -> Result<PreProcessingInfo, PreProcessingError>
where
  P: AsRef<Path>,
{
  let shader_source = fs::read_to_string(&shader_file).map_err(|e| PreProcessingError::IO {
    error: e,
    file: shader_file.as_ref().to_path_buf(),
  })?;

  let mut info = PreProcessingInfo {
    included_files: Default::default(),
    source_code: String::new(),
    no_standalone: false,
  };
  for line in shader_source.lines() {
    if line.starts_with(NO_STANDALONE) {
      info.no_standalone = true;
      continue;
    }

    if let Some(include) = line.strip_prefix("//include ") {
      let include_file = shader_file.as_ref().parent().unwrap().join(include);
      info.source_code += &pre_process_shader(include_file)?.source_code;
      info.included_files.insert(include.to_string());
      continue;
    }

    info.source_code += &format!("{}\n", line);
  }

  Ok(info)
}

#[derive(Debug)]
struct PreProcessingInfo {
  pub included_files: HashSet<String>,
  pub source_code: String,
  pub no_standalone: bool,
}

#[allow(dead_code)]
#[derive(Debug)]
enum PreProcessingError {
  IO { error: io::Error, file: PathBuf },
}
