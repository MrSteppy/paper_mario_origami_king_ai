use crate::type_analysis::member::Member;
use crate::type_analysis::primitive_type::PrimitiveType;
use std::fmt::{Display, Formatter};

///Describes a field in a [`MemoryLayout`]
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[deprecated]
pub struct PrimitiveMember {
  pub name: String,
  pub r#type: PrimitiveType,
}

impl PrimitiveMember {
  pub fn member_name_for_index(index: usize) -> String {
    format!("_{index}")
  }

  pub fn new<S, P>(name: S, r#type: P) -> Self
  where
    S: ToString,
    P: Into<PrimitiveType>,
  {
    Self {
      name: name.to_string(),
      r#type: r#type.into(),
    }
  }
}

impl Display for PrimitiveMember {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}: {}", self.name, self.r#type)
  }
}

///Describes how a [`PrimitiveComposition`] will be lied out in memory
pub struct MemoryLayout {
  pub primitive_members: Vec<Member<PrimitiveType>>,
  pub number_of_padding_bytes: usize,
}

impl Display for MemoryLayout {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "[{}]",
      self
        .primitive_members
        .iter()
        .map(|member| member.to_string())
        .chain(
          Some(self.number_of_padding_bytes)
            .iter()
            .filter(|&&b| b > 0)
            .map(|b| format!("+{b} padding bytes"))
        )
        .collect::<Vec<_>>()
        .join(", ")
    )
  }
}

#[cfg(test)]
mod test_memory_layout_creation {
  use crate::primitive_composition::composite_type::CompositeType;
  use crate::primitive_composition::PrimitiveComposition;
  use crate::type_analysis::member::Member;
  use crate::type_analysis::primitive_type::PrimitiveType;

  #[test]
  fn test_create_memory_layout() {
    let vec4_type = PrimitiveType::new_aligned("vec4<f32>", 16, 16, "glam::Vec4").unwrap();
    let vec3_type = PrimitiveType::new("vec3<f32>", 12, "glam::Vec3");

    let composition = PrimitiveComposition::from(
      CompositeType::new("Vertex")
        .with_member(Member::new("position", vec3_type.clone()))
        .with_member(Member::new("color", vec4_type.clone())),
    );
    let layout = composition.create_memory_layout();
    assert_eq!(
      Member::new("_1", vec4_type.clone()),
      layout.primitive_members[0]
    );
    assert_eq!(
      Member::new("_0", vec3_type.clone()),
      layout.primitive_members[1]
    );
    assert_eq!(4, layout.number_of_padding_bytes);
  }
}
