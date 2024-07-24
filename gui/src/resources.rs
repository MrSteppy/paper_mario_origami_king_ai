use std::error::Error;
use std::fmt::{Display, Formatter};

use image::ImageError;
use winit::window::{BadIcon, Icon};

pub fn load_icon(bytes: &[u8]) -> Result<Icon, IconError> {
  let rgba = image::load_from_memory(bytes)?.to_rgba8();
  let (width, height) = rgba.dimensions();
  let icon = Icon::from_rgba(rgba.to_vec(), width, height)?;
  Ok(icon)
}

#[derive(Debug)]
pub enum IconError {
  Image(ImageError),
  BadIcon(BadIcon),
}

impl From<ImageError> for IconError {
  fn from(value: ImageError) -> Self {
    Self::Image(value)
  }
}

impl From<BadIcon> for IconError {
  fn from(value: BadIcon) -> Self {
    Self::BadIcon(value)
  }
}

impl Display for IconError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        IconError::Image(e) => e.to_string(),
        IconError::BadIcon(e) => e.to_string(),
      }
    )
  }
}

impl Error for IconError {}

macro_rules! resource_path {
  ($($sub_directory: ident)* $file_name: literal) => {
    concat!(
      env!("CARGO_MANIFEST_DIR"),
      env!("PATH_SEPARATOR"),
      "resources",
      $( env!("PATH_SEPARATOR"), stringify!($sub_directory), )*
      env!("PATH_SEPARATOR"),
      $file_name
    )
  };
}
pub(crate) use resource_path;

///Includes a file from the `resources` folder as string
///
/// <strong>Example<strong>
/// ```
/// include_resource_str!(test "test.txt") //includes resources/test/test.txt
/// ```
macro_rules! include_resource_str {
  ($($sub_directory: ident)* $file_name: literal) => {
    include_str!(crate::resources::resource_path!($($sub_directory)* $file_name))
  };
}
pub(crate) use include_resource_str;

///Includes a file from the `resources` folder as bytes
///
/// <strong>Example<strong>
/// ```rust
/// include_resource_bytes!(test "test.txt") //includes resources/test/test.txt
/// ```
macro_rules! include_resource_bytes {
  ($($sub_directory: ident)* $file_name: literal) => {
    include_bytes!(crate::resources::resource_path!($($sub_directory)* $file_name))
  };
}
pub(crate) use include_resource_bytes;

// macro_rules! test {
//   ($($path: ident/)* $file: ident.$ext:ident) => {
//     concat!(stringify!($file), ".", stringify!($ext))
//   };
// }
//
// fn test() {
//   let s = test!(hewo.exe);
// }

#[cfg(test)]
mod test_include {
  #[test]
  fn test_include_str() {
    assert_eq!("foo bar", include_resource_str!(test "test.txt"));
  }
}
