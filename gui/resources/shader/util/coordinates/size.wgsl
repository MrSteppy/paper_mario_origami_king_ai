//once 

struct Size {
  width: u32,
  height: u32,
}

fn Size_from(v: vec2<u32>) -> Size {
  return Size(v.x, v.y);
}