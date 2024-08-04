use std::collections::HashMap;

use crate::primitive_composition::PrimitiveComposition;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct PreProcessingEnvironment {
  primitive_and_native_types: HashMap<String, PrimitiveComposition>,
}

///A native type is a type which is native in wgsl but can not be translated by wgsl_to_wgpu, like mat4x4<f32>.
/// Every type added which is not a [`PrimitiveType`] will be considered native.  
impl PreProcessingEnvironment {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn with<T>(mut self, r#type: T) -> Self
  where
    T: Into<PrimitiveComposition>,
  {
    self.add(r#type);
    self
  }

  pub fn add<T>(&mut self, r#type: T)
  where
    T: Into<PrimitiveComposition>,
  {
    let r#type = r#type.into();
    self.primitive_and_native_types.insert(
      match &r#type {
        PrimitiveComposition::Primitive(primitive) => &primitive.name,
        PrimitiveComposition::Composite(native) => &native.name,
      }
      .to_owned(),
      r#type,
    );
  }

  pub fn types(&self) -> &HashMap<String, PrimitiveComposition> {
    &self.primitive_and_native_types
  }
}
