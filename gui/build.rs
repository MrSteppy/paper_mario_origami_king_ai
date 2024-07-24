use std::{env, fs};
use std::path::Path;

use wgsl_to_wgpu::{create_shader_module, MatrixVectorTypes, WriteOptions};

fn main() {
  println!("cargo::rerun-if-changed=resources/shader");
  println!("cargo::rerun-if-changed=src/shader.rs");
  println!("cargo::rerun-if-changed=src/shader");

  let shader_module_directory = Path::new("src/shader");
  fs::create_dir_all(shader_module_directory).expect("failed to create shader module directory");
  let mut created_shader_modules = vec![];

  for entry in fs::read_dir("resources/shader")
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
      let shader_source = fs::read_to_string(&path).expect("failed to read shader file");
      let shader_module_source = create_shader_module(
        &shader_source,
        Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
          .join(&path)
          .to_str()
          .expect("invalid path to shader src"),
        WriteOptions {
          derive_bytemuck_vertex: true,
          derive_bytemuck_host_shareable: true,
          derive_encase_host_shareable: true,
          derive_serde: false,
          matrix_vector_types: MatrixVectorTypes::Glam,
        },
      )
      .expect("failed to create shader module");

      fs::write(
        Path::new("src/shader").join(format!("{}.rs", shader_name)),
        shader_module_source,
      )
      .expect("failed to save shader module");
      created_shader_modules.push(shader_name);
    }
  }

  fs::write(
    "src/shader.rs",
    format!("#[allow(clippy::module_inception)]\n\n{}",
    created_shader_modules
      .iter()
      .map(|mod_name| format!("pub mod {};", mod_name))
      .collect::<Vec<_>>()
      .join("\n")),
  )
  .expect("failed to create module shader.rs");
}
