pub trait NamedType {
  fn name(&self) -> &str;
  fn rust_equivalent(&self) -> Option<&str>;
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct NamedTypeParent {
  pub name: String,
  pub rust_equivalent: Option<String>,
}

impl NamedTypeParent {
  pub fn new<S>(name: S) -> Self
  where
    S: ToString,
  {
    Self {
      name: name.to_string(),
      rust_equivalent: None,
    }
  }

  pub fn with_rust_equivalent<R>(mut self, rust_equivalent: R) -> Self
  where
    R: ToString,
  {
    self.rust_equivalent = Some(rust_equivalent.to_string());
    self
  }
}

impl NamedType for NamedTypeParent {
  fn name(&self) -> &str {
    &self.name
  }

  fn rust_equivalent(&self) -> Option<&str> {
    self.rust_equivalent.as_ref().map(|s| s.as_ref())
  }
}