//once

struct Pixel {
  x: u32,
  y: u32,
}

fn Pixel_from(v: vec2<u32>) -> Pixel {
  return Pixel(v.x, v.y);
}