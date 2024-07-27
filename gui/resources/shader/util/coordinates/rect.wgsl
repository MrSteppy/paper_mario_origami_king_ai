//once

//include pixel.wgsl

struct Rect {
  top_left: Pixel,
  bottom_right: Pixel,
}

fn Rect_from(v: vec4<u32>) -> Rect {
  return Rect(Pixel_from(v.xy), Pixel_from(v.zw));
}