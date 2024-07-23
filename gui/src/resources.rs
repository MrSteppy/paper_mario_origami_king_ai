#[cfg(not(target_os = "windows"))]
macro_rules! path_separator {
  () => {
    "/"
  };
}
#[cfg(target_os = "windows")]
macro_rules! path_separator {
  () => {
    "\\"
  };
}
pub(crate) use path_separator;

macro_rules! resource_path {
  ($($sub_directory: ident)* $file_name: literal) => {
    concat!(
      env!("CARGO_MANIFEST_DIR"),
      crate::resources::path_separator!(),
      "resources",
      $( crate::resources::path_separator!(), stringify!($sub_directory), )*
      crate::resources::path_separator!(),
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

#[cfg(test)]
mod test_include {
  #[test]
  fn test_include_str() {
    assert_eq!("foo bar", include_resource_str!(test "test.txt"));
  }
}