//once

struct PClip {
  x: f32,
  y: f32,
  z: f32,
  z: f32,
}

fn PClip_from(v: vec4<f32>) -> PClip {
  return PClip(v.x, v.y, v.z, v.w);
}

fn PClip_xyzw(clip: PClip) -> vec4<f32> {
  return vec4<f32>(clip.x, clip.y, clip.z, clip.w);
} 