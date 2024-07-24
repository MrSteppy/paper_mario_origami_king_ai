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
  ($first:ident $(/$path: ident)*.$ext:ident) => {
    concat!(
      env!("CARGO_MANIFEST_DIR"),
      env!("PATH_SEPARATOR"),
      "resources",
      env!("PATH_SEPARATOR"),
      stringify!($first),
      $( env!("PATH_SEPARATOR"), stringify!($path), )*
      ".",
      stringify!($ext)
    )
  };
}
pub(crate) use resource_path;

///Includes a file from the `resources` folder as string
///
/// <strong>Example<strong>
/// ```
/// include_resource_str!(test/test.txt) //includes resources/test/test.txt
/// ```
macro_rules! include_resource_str {
  ($first:ident $(/$path: ident)*.$ext:ident) => {
    include_str!(crate::resources::resource_path!($first $(/$path)*.$ext))
  };
}
pub(crate) use include_resource_str;

///Includes a file from the `resources` folder as bytes
///
/// <strong>Example<strong>
/// ```rust
/// include_resource_bytes!(test/test.txt) //includes resources/test/test.txt
/// ```
macro_rules! include_resource_bytes {
  ($first:ident $(/$path: ident)*.$ext:ident) => {
    include_bytes!(crate::resources::resource_path!($first $(/$path)*.$ext))
  };
}
pub(crate) use include_resource_bytes;

#[cfg(test)]
mod test_include {
  #[test]
  fn test_include_str() {
    assert_eq!("foo bar", include_resource_str!(test/test_data.txt));
  }
}
