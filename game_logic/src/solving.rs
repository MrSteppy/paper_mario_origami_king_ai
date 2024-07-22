use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use enum_assoc::Assoc;

use crate::arena::{Arena, ToArenaSymbol};
use crate::position::{Move, Num, Position};
use crate::position::Dimension::{Column, Row};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SolvableArena {
  pub inner: Arena<Enemy>,
  pub num_groups: Option<Num>,
  pub throwing_hammer_available: bool,
}

impl SolvableArena {
  pub fn num_groups(&self) -> Num {
    self.num_groups.unwrap_or_else(|| {
      let num_enemies = self.enemies.len();
      let mut num_groups = num_enemies / 4;
      if num_enemies % 4 != 0 {
        num_groups += 1;
      }

      num_groups as Num
    })
  }

  pub fn is_solved(&self) -> bool {
    Coverage::find(self).is_some()
  }
}

impl Deref for SolvableArena {
  type Target = Arena<Enemy>;

  fn deref(&self) -> &Self::Target {
    &self.inner
  }
}

impl DerefMut for SolvableArena {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.inner
  }
}

impl Default for SolvableArena {
  fn default() -> Self {
    Self {
      inner: Default::default(),
      num_groups: None,
      throwing_hammer_available: true,
    }
  }
}

#[derive(Debug, Clone, Default)]
pub struct Coverage {
  areas: Vec<EnemyArea>,
}

impl Coverage {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn find(arena: &SolvableArena) -> Option<Self> {
    let mut coverage = Self::new();
    let groups = arena.num_groups() as usize;

    //top enemies must be covered by long area
    for top_enemy in arena
      .enemies
      .iter()
      .filter(|enemy| enemy.row >= Row.size() / 2)
    {
      if top_enemy.weakness == Some(Attack::Hammer) && !arena.throwing_hammer_available {
        return None;
      }
      if let Some(area) = coverage.get_covering_area_mut(top_enemy) {
        //make sure area attack matches enemy weakness
        if let Some(weakness) = top_enemy.weakness {
          if let Some(attack) = area.attack {
            if weakness != attack {
              return None;
            }
          } else {
            area.attack = Some(weakness);
          }
        }
      } else if coverage.len() < groups {
        coverage.push(EnemyArea {
          target_area: TargetArea::Long {
            column: top_enemy.column,
          },
          attack: top_enemy.weakness,
        })
      } else {
        return None;
      }
    }

    Self::find_bottom_coverage(coverage, arena, groups)
  }

  fn find_bottom_coverage<G>(
    mut previous_coverage: Coverage,
    arena: &SolvableArena,
    num_groups: G,
  ) -> Option<Self>
  where
    G: Into<Option<usize>>,
  {
    let groups = num_groups
      .into()
      .unwrap_or_else(|| arena.num_groups() as usize);
    for bottom_enemy in arena
      .enemies
      .iter()
      .filter(|enemy| enemy.row < Row.size() / 2)
    {
      if let Some(area) = previous_coverage.get_covering_area_mut(bottom_enemy) {
        //make sure area attack matches enemy weakness
        if let Some(weakness) = bottom_enemy.weakness {
          if let TargetArea::Long {..} = area.target_area {
            if weakness == Attack::Hammer && !arena.throwing_hammer_available {
              return None;
            }
          }

          if let Some(attack) = area.attack {
            if weakness != attack {
              return None;
            }
          } else {
            area.attack = Some(weakness);
          }
        }
      } else {
        //jump enemies can't be covered by wide attacks
        if bottom_enemy.weakness == Some(Attack::Jump) {
          return None;
        }

        return if previous_coverage.len() < groups {
          let left_area = EnemyArea {
            target_area: TargetArea::Wide {
              left_column: (bottom_enemy.column + Column.size() - 1) % Column.size(),
            },
            attack: bottom_enemy.weakness,
          };
          if previous_coverage.can_hold(&left_area) {
            let mut coverage = previous_coverage.clone();
            coverage.push(left_area);
            if let Some(coverage) = Self::find_bottom_coverage(coverage, arena, groups) {
              return Some(coverage);
            }
          }
          let right_area = EnemyArea {
            target_area: TargetArea::Wide {
              left_column: bottom_enemy.column,
            },
            attack: bottom_enemy.weakness,
          };
          if previous_coverage.can_hold(&right_area) {
            let mut coverage = previous_coverage.clone();
            coverage.push(right_area);
            return Self::find_bottom_coverage(coverage, arena, groups);
          }
          None
        } else {
          None
        };
      }
    }

    Some(previous_coverage)
  }

  pub fn can_hold(&self, area: &TargetArea) -> bool {
    let mut covered_columns = HashSet::new();
    for area in &self.areas {
      match area.target_area {
        TargetArea::Long { column } => {
          covered_columns.insert(column);
        }
        TargetArea::Wide { left_column } => {
          covered_columns.insert(left_column);
          covered_columns.insert(TargetArea::right_column(left_column));
        }
      }
    }
    !match area {
      TargetArea::Long { column } => covered_columns.contains(column),
      TargetArea::Wide { left_column } => {
        covered_columns.contains(left_column)
          || covered_columns.contains(&Column.next(*left_column))
      }
    }
  }

  pub fn get_covering_area_mut(&mut self, position: &Position) -> Option<&mut EnemyArea> {
    self.areas.iter_mut().find(|area| area.covers(position))
  }

  pub fn get_covering_area(&self, position: &Position) -> Option<&EnemyArea> {
    self.areas.iter().find(|area| area.covers(position))
  }

  pub fn covers(&self, position: &Position) -> bool {
    self.get_covering_area(position).is_some()
  }
}

impl Deref for Coverage {
  type Target = Vec<EnemyArea>;

  fn deref(&self) -> &Self::Target {
    &self.areas
  }
}

impl DerefMut for Coverage {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.areas
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct EnemyArea {
  pub target_area: TargetArea,
  pub attack: Option<Attack>,
}

impl Deref for EnemyArea {
  type Target = TargetArea;

  fn deref(&self) -> &Self::Target {
    &self.target_area
  }
}

impl DerefMut for EnemyArea {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.target_area
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TargetArea {
  Long { column: Num },
  Wide { left_column: Num },
}

impl TargetArea {
  pub fn right_column(left_column: Num) -> Num {
    (left_column + 1) % Column.size()
  }

  pub fn covers(&self, position: &Position) -> bool {
    match self {
      TargetArea::Long { column } => position.column == *column,
      TargetArea::Wide { left_column } => {
        position.row < Row.size() / 2
          && (position.column == *left_column
            || position.column == (*left_column + 1) % Column.size())
      }
    }
  }
}

impl Display for TargetArea {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      TargetArea::Long { column } => write!(f, "c{}", column + 1),
      TargetArea::Wide { left_column } => {
        write!(f, "h{}{}", left_column + 1, Column.next(*left_column) + 1)
      }
    }
  }
}

//TODO add option to interrupt
pub fn solve<'a, C>(arena: &SolvableArena, in_turns: Num, fast: bool, arena_solved_cache: C) -> Option<Vec<Move>>
where
  C: Into<Option<&'a mut HashMap<SolvableArena, bool>>>,
{
  let mut new_cache = HashMap::new();
  let cache = match arena_solved_cache.into() {
    Some(cache) => cache,
    None => &mut new_cache,
  };

  if let Some(solved) = cache.get(arena) {
    if *solved {
      return Some(vec![]);
    }
  } else if arena.is_solved() {
    cache.insert(arena.clone(), true);
    return Some(vec![]);
  } else {
    cache.insert(arena.clone(), false);
  }

  if in_turns == 0 {
    return None;
  }

  let mut best_solution: Option<Vec<Move>> = None;
  for dimension in [Row, Column] {
    for coordinate in 0..dimension.size() {
      for amount in 1..dimension.changes().size() {
        let move_ = Move::new(dimension, coordinate, amount, true).unwrap();
        let mut arena_clone = arena.clone();
        arena_clone.apply_move(move_);

        if let Some(mut solution) = solve(&arena_clone, in_turns - 1, fast, &mut *cache) {
          solution.insert(0, move_);

          if fast {
            return Some(solution)
          }

          if let Some(current_best) = &best_solution {
            //solution is better if it is shorter and has a lower sum of absolute shortest amounts
            match solution.len().cmp(&current_best.len()) {
              Ordering::Less => {
                best_solution = Some(solution);
              }
              Ordering::Equal => {
                if solution.iter().map(|m| m.normalized().amount).sum::<Num>()
                  < current_best.iter().map(|m| m.normalized().amount).sum()
                {
                  best_solution = Some(solution);
                }
              }
              Ordering::Greater => {}
            }
          } else {
            best_solution = Some(solution);
          }
        }
      }
    }
  }

  best_solution
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Enemy {
  pub position: Position,
  pub weakness: Option<Attack>,
}

impl Deref for Enemy {
  type Target = Position;

  fn deref(&self) -> &Self::Target {
    &self.position
  }
}

impl DerefMut for Enemy {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.position
  }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Assoc)]
#[func(pub fn symbol(& self) -> char)]
pub enum Attack {
  ///attack enemies with a hammer
  #[assoc(symbol = 'H')]
  Hammer,
  ///attack enemies with a jump
  #[assoc(symbol = 'J')]
  Jump,
}

impl ToArenaSymbol for Enemy {
  fn to_arena_symbol(&self) -> char {
    if let Some(weakness) = self.weakness {
      weakness.symbol()
    } else  {
      'E'
    }
  }
}

#[cfg(test)]
mod test_coverage {
  use crate::parse;
  use crate::solving::{Coverage, SolvableArena};

  #[test]
  fn test_solved() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 1234", "c4 12", "c5 12"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_some())
  }

  #[test]
  fn test_unsolved() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 124", "c3 3", "c4 12", "c5 12"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_none())
  }

  #[test]
  fn test_too_few_enemies() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 124", "c4 12", "c5 1"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_some())
  }

  #[test]
  fn test_ninja_covered() {
    let mut arena = SolvableArena::default();
    parse(&mut arena, "c2 1234 J").expect("parse error");

    assert!(Coverage::find(&arena).is_some());
  }

  #[test]
  fn test_ninja_uncovered() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 12 J", "c3 12 J"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_none());
  }

  #[test]
  fn test_no_hammer() {
    let mut arena = SolvableArena {
      throwing_hammer_available: false,
      ..Default::default()
    };
    for cmd in ["c4 1 H", "c4 23"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    Coverage::find(&arena).ok_or(()).expect_err("no coverage should exists");
  }
}

#[cfg(test)]
mod test_solve {
  use crate::parse;
  use crate::position::Move;
  use crate::solving::{SolvableArena, solve};

  #[test]
  fn test_simple_solve() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 124", "c3 3"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    let solution = solve(&arena, 1, false, None).expect("is solvable");
    assert_eq!("r3 -1", steps(&solution));
  }

  #[test]
  fn test_two_steps() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 124", "c3 3", "c4 2", "c5 123"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    let solution = solve(&arena, 2, false, None).expect("is solvable");
    assert_eq!("r3 -1, c4 -1", steps(&solution));
  }

  fn steps(moves: &[Move]) -> String {
    moves
      .iter()
      .map(|m| m.to_string())
      .collect::<Vec<_>>()
      .join(", ")
  }
}