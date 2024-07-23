struct VertexOutput {
  @builtin(position) clip_position: vec4<f32>, //since clip position is a builtin position, it can't be accessed later
  @location(0) vert_position: vec3<f32>,
};

@vertex
fn vs_main(
  @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
  //use index 0, 1, 2 to create the corners of a triangle each
  var out: VertexOutput; //var: mut, type required
  let x = f32(1 - i32(vertex_index)) * 0.5; //let: non-mut, type infered
  let y = f32(i32(vertex_index & 1u) * 2 - 1) * 0.5;
  out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
  out.vert_position = out.clip_position.xyz;
  return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
  let r = clip_range_to_color_range(in.vert_position[0]);
  let g = clip_range_to_color_range(in.vert_position[1]);
  let b = clip_range_to_color_range(in.vert_position[2]);
  return vec4<f32>(r, g, b, 1.0);
}

fn clip_range_to_color_range(clip_value: f32) -> f32 {
  //-1..1 => 0..1
  return (clip_value + 1.0) / 2.0;
}