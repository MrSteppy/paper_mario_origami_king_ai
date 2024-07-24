struct VertexInput {
  @location(0) position: vec3<f32>,
  @location(1) _padding: f32,
  @location(2) color: vec4<f32>,
}

struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>, //since clip position is a builtin position, it can't be accessed later
  @location(0) position: vec3<f32>,
  @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
  var out: VertexOutput;
  out.clip_position = vec4<f32>(in.position, 1.0);
  out.position = in.position;
  out.color = in.color;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  return in.color;
}

fn eq(x: f32, y: f32) -> bool {
  return x - y < 0.001;
}