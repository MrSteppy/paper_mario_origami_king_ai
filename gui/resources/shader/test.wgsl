#no-standalone

struct Vertex {
  @location(0) tex_rect: mat4x4<f32>,
}

@vertex
fn vs_main(in: Vertex) -> @builtin(position) vec4<f32> {
  return vec4<f32>();
}