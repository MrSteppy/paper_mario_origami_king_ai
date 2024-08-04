use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;

use crate::declaration::Declaration;
use crate::struct_layout::StructLayout;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PreProcessingCache {
  pub includes: HashSet<PathBuf>,
  struct_layouts: HashMap<String, Declaration<StructLayout>>,
}

impl PreProcessingCache {
  pub fn structs(&self) -> &HashMap<String, Declaration<StructLayout>> {
    &self.struct_layouts
  }

  pub fn structs_mut(&mut self) -> &mut HashMap<String, Declaration<StructLayout>> {
    &mut self.struct_layouts
  }

  ///inserts a [`Declaration`] in the cache and returns the previous [`Declaration`], if present
  pub fn insert<S>(&mut self, declaration: Declaration<S>) -> Option<Declaration<StructLayout>>
  where
    S: Into<StructLayout>,
  {
    let declaration = declaration.convert(|info, s| info + s.into());
    self
      .struct_layouts
      .insert(declaration.declared.name().to_string(), declaration)
  }

  pub fn update<S>(
    &mut self,
    layout: S,
  ) -> Result<&mut Declaration<StructLayout>, MissingDeclarationError>
  where
    S: Into<StructLayout>,
  {
    let layout = layout.into();
    let declaration = self
      .struct_layouts
      .get_mut(layout.name())
      .ok_or(MissingDeclarationError)?;
    declaration.declared = layout;
    Ok(declaration)
  }
}

#[derive(Debug)]
pub struct MissingDeclarationError;

impl Display for MissingDeclarationError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "LayoutDeclaration has not been inserted yet")
  }
}

impl Error for MissingDeclarationError {}
