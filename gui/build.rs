use std::fs;
use std::path::Path;
use std::process::Command;

use wgsl_to_wgpu::{create_shader_module, MatrixVectorTypes, WriteOptions};

use shader_pre_processor::environment::PreProcessingEnvironment;
use shader_pre_processor::pre_processing_cache::PreProcessingCache;
use shader_pre_processor::type_analysis::primitive_type::PrimitiveType;
use shader_pre_processor::{pre_process_shader, ProcessContext};

const INCLUDE_HOOK_POINT: &str = "INCLUDE_HOOK_POINT";

fn main() {
  println!("cargo::rerun-if-changed=resources/shader/**");

  let environment = PreProcessingEnvironment::new()
    .with(PrimitiveType::new("f32", 4, "f32"))
    .with(PrimitiveType::new("u32", 4, "u32"));

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

      if let Some(source_code) = pre_process_shader(
        &path,
        ProcessContext::Standalone,
        &mut PreProcessingCache::default(),
        &environment,
      )
      .expect("failed to pre-process shader")
      {
        let shader_module_source = create_shader_module(
          &source_code,
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
          &format!("include_str!(\"{INCLUDE_HOOK_POINT}\")"),
          &format!("r#\"\n{source_code}\"#"),
        );

        shader_rs_source += &format!("pub mod {shader_name} {{\n{shader_module_source}\n}}\n");

        println!("Ok!");
      } else {
        println!("No standalone - ignoring");
      }
    }
  }

  let shader_rs_path = Path::new("src").join("shader.rs");
  fs::write(&shader_rs_path, shader_rs_source).expect("failed to create shader.rs");
  //try running rust fmt on the file
  if let Ok(mut process) = Command::new("rustfmt").arg(shader_rs_path).spawn() {
    let _ = process.wait();
  }
}
