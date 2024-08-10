use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Member<T> {
  pub annotation_values: Vec<String>,
  pub name: String,
  pub r#type: T,
}

impl<T> Member<T> {
  pub fn new<S>(name: S, r#type: T) -> Self
  where
    S: ToString
  {
    Self {
      name: name.to_string(),
      r#type,
      annotation_values: vec![],
    }
  }

  pub fn new_annotated<A, S>(annotations: &[A], name: S, r#type: T) -> Self
  where
    A: ToString,
    S: ToString
  {
    Self {
      name: name.to_string(),
      r#type,
      annotation_values: annotations.iter().map(|v| v.to_string()).collect(),
    }
  }

  pub fn convert<R, F>(self, type_mapping: F) -> Member<R>
  where
    F: FnOnce(T) -> R,
  {
    Member {
      annotation_values: self.annotation_values,
      name: self.name,
      r#type: type_mapping(self.r#type),
    }
  }

  pub fn convert_into<R>(self) -> Member<R>
  where
    T: Into<R>,
  {
    self.convert(|t| t.into())
  }
}

impl<T> Display for Member<T>
where
  T: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}{}: {}",
      self
        .annotation_values
        .iter()
        .map(|value| format!("@{value} "))
        .collect::<Vec<_>>()
        .join(""),
      self.name,
      self.r#type
    )
  }
}