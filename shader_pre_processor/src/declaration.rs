use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Declaration<T> {
  pub info: DeclarationInfo,
  pub declared: T,
}

impl<T> Declaration<T> {
  pub fn new<I, D>(info: I, declared: D) -> Self
  where
    I: Into<DeclarationInfo>,
    D: Into<T>,
  {
    Self {
      info: info.into(),
      declared: declared.into(),
    }
  }

  pub fn separate(self) -> (DeclarationInfo, T) {
    (self.info, self.declared)
  }

  pub fn convert<F, R>(self, mapping: F) -> R
  where
    F: FnOnce(DeclarationInfo, T) -> R,
  {
    mapping(self.info, self.declared)
  }
}

impl<T> Display for Declaration<T>
where
  T: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.info, self.declared)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SourceLocation {
  pub source_file: PathBuf,
  pub line_nr: usize,
}

impl From<&Path> for SourceLocation {
  fn from(value: &Path) -> Self {
    Self::new(value)
  }
}

impl SourceLocation {
  pub fn new<P>(source_file: P) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      source_file: source_file.as_ref().to_path_buf(),
      line_nr: 0,
    }
  }

  pub fn at<P>(source_file: P, line_nr: usize) -> Self
  where
    P: AsRef<Path>,
  {
    Self {
      source_file: source_file.as_ref().to_path_buf(),
      line_nr,
    }
  }
}

impl Add<usize> for SourceLocation {
  type Output = Self;

  fn add(mut self, rhs: usize) -> Self::Output {
    self.line_nr += rhs;
    self
  }
}

impl AddAssign<usize> for SourceLocation {
  fn add_assign(&mut self, rhs: usize) {
    self.line_nr += rhs;
  }
}

impl Display for SourceLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}:{}", self.source_file, self.line_nr)
  }
}

#[non_exhaustive]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DeclarationInfo {
  pub source_location: SourceLocation,
}

impl DeclarationInfo {
  pub fn new<S>(source_location: S) -> Self
  where
    S: Into<SourceLocation>,
  {
    Self {
      source_location: source_location.into(),
    }
  }

  pub fn with<T>(self, declared: T) -> Declaration<T> {
    Declaration::new(self.clone(), declared)
  }
}

impl<T> Add<T> for DeclarationInfo {
  type Output = Declaration<T>;

  fn add(self, rhs: T) -> Self::Output {
    self.with(rhs)
  }
}

impl Display for DeclarationInfo {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "at {}", self.source_location)
  }
}
