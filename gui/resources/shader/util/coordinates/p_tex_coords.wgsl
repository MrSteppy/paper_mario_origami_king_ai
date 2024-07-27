//once

struct PTexCoords {
  x: f32,
  y: f32,
}

fn PTexCoords_from(v: vec2<f32>) -> PTexCoords {
  return PTexCoords(v.x, v.y);
}