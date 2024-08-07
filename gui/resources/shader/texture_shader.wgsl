#include util/coordinates.wgsl

struct Vertex {
  @location(0) position: vec3<f32>,
  @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>,
  @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(in: Vertex) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = vec4<f32>(in.position, 1.0);
  out.tex_coords = in.tex_coords;
  return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var t_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, t_sampler, in.tex_coords);
}