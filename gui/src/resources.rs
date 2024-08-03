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

#[macro_export]
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

///Includes a file from the `resources` folder as string
///
/// <strong>Example<strong>
/// ```
/// use gui::include_resource_str;
/// 
/// let data = include_resource_str!(test/test_data.txt); //includes resources/test/test_data.txt
/// ```
#[macro_export]
macro_rules! include_resource_str {
  ($first:ident $(/$path: ident)*.$ext:ident) => {
    include_str!($crate::resource_path!($first $(/$path)*.$ext))
  };
}

///Includes a file from the `resources` folder as bytes
///
/// <strong>Example<strong>
/// ```rust
/// use gui::include_resource_bytes;
/// 
/// let bytes = include_resource_bytes!(test/test_data.txt); //includes resources/test/test_data.txt
/// ```
#[macro_export]
macro_rules! include_resource_bytes {
  ($first:ident $(/$path: ident)*.$ext:ident) => {
    include_bytes!($crate::resource_path!($first $(/$path)*.$ext))
  };
}

#[cfg(test)]
mod test_include {
  #[test]
  fn test_include_str() {
    assert_eq!("foo bar", include_resource_str!(test/test_data.txt));
  }
}
