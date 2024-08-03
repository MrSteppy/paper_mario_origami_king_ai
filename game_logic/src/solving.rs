use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};

use enum_assoc::Assoc;

use crate::arena::{Arena, ToArenaSymbol};
use crate::position::{Move, Num, Position};
use crate::position::Dimension::{Column, Row};

#[derive(Debug, Clone, Eq, PartialEq, Hash, Default)]
pub struct SolvableArena {
  pub inner: Arena<Enemy>,
  pub num_groups: Option<Num>,
  pub available_equipment: AvailableEquipment,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct AvailableEquipment {
  pub throwing_hammer: bool,
  pub iron_boots: bool,
}

impl Default for AvailableEquipment {
  fn default() -> Self {
    Self {
      throwing_hammer: true,
      iron_boots: true,
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

  pub fn find<A>(arena: A) -> Option<Self>
  where
    A: Borrow<SolvableArena>,
  {
    let arena = arena.borrow();
    let mut enemies: Vec<_> = arena
      .enemies
      .iter()
      .map(|enemy| (enemy, RingPosition::from(&enemy.position)))
      .collect();
    enemies.sort_by_cached_key(|(_, ring_pos)| *ring_pos);
    Self::default().finalize(enemies.into_iter(), arena, arena.num_groups() as usize)
  }

  fn finalize<'a, E>(
    mut self,
    mut enemy_iterator: E,
    arena: &'a SolvableArena,
    num_groups: usize,
  ) -> Option<Self>
  where
    E: Iterator<Item = (&'a Enemy, RingPosition)> + Clone,
  {
    let equipment = &arena.available_equipment;
    //cover all enemies which are in the outer rings
    while let Some((enemy, ring_position)) = enemy_iterator.next() {
      match ring_position {
        RingPosition::Outer => {
          //hammer is not available on outer rings
          if let Some(required_attack) = &enemy.required_attack {
            let equipment_present = match required_attack {
              RequiredAttack::IronBootsOrHammer => {
                equipment.iron_boots || equipment.throwing_hammer
              }
              RequiredAttack::Hammer => equipment.throwing_hammer,
              RequiredAttack::Jump => true,
            };
            if !equipment_present {
              return None;
            }
          }

          //check if enemy is already covered
          if let Some(covering_area) = self.get_covering_area_mut(enemy) {
            covering_area.limit_attacks(enemy).ok()?;
            continue;
          }

          //check if another group is available
          if self.len() >= num_groups {
            return None;
          }

          self.push(EnemyArea::long(enemy));
        }
        RingPosition::Inner => {
          //check if enemy is already covered
          if let Some(covering_area) = self.get_covering_area_mut(enemy) {
            //enemies which require a hammer covered by long areas require a throwing hammer
            if !Self::hammer_enemy_can_be_covered(enemy, covering_area, equipment) {
              return None;
            }

            covering_area.limit_attacks(enemy).ok()?;
            continue;
          }

          //check if another group is available
          if self.len() >= num_groups {
            return None;
          }

          //jump enemies can't be covered by wide areas (hammer only)
          let long_area_required = enemy.required_attack == Some(RequiredAttack::Jump);

          macro_rules! try_finalize_with {
            ($area: expr) => {
              let mut next_coverage = self.clone();
              next_coverage.push($area);
              if let Some(finalized) =
                next_coverage.finalize(enemy_iterator.clone(), arena, num_groups)
              {
                return Some(finalized);
              }
            };
          }

          if !long_area_required {
            //try left-bound wide area
            let left_area = EnemyArea::wide(enemy, true);
            if self.can_hold(&left_area) {
              try_finalize_with!(left_area);
            }

            //try right-bound wide area
            let right_area = EnemyArea::wide(enemy, false);
            if self.can_hold(&right_area) {
              try_finalize_with!(right_area);
            }
          }

          //try long area
          let long_area = EnemyArea::long(enemy);
          if Self::hammer_enemy_can_be_covered(enemy, &long_area, equipment) {
            try_finalize_with!(long_area);
          }

          return None;
        }
      }
    }

    Some(self)
  }

  fn hammer_enemy_can_be_covered(
    enemy: &Enemy,
    covering_area: &EnemyArea,
    equipment: &AvailableEquipment,
  ) -> bool {
    //enemies which require a hammer covered by long areas require a throwing hammer
    !matches!((
      &covering_area.target_area,
      &enemy.required_attack,
      equipment.throwing_hammer,
    ), (TargetArea::Long { .. }, Some(RequiredAttack::Hammer), false))
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

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash)]
enum RingPosition {
  Outer,
  Inner,
}

impl From<&Position> for RingPosition {
  fn from(value: &Position) -> Self {
    if value.row >= Row.size() / 2 {
      Self::Outer
    } else {
      Self::Inner
    }
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EnemyArea {
  pub target_area: TargetArea,
  pub attack_whitelist: Option<Vec<Attack>>,
}

impl EnemyArea {
  pub fn long(enemy: &Enemy) -> Self {
    let mut res = Self::new(TargetArea::long(enemy));
    let _ = res.limit_attacks(enemy);
    res
  }

  pub fn wide(enemy: &Enemy, left_bound: bool) -> Self {
    let mut res = Self::new(TargetArea::wide(enemy, left_bound));
    let _ = res.limit_attacks(enemy);
    res
  }

  pub fn new(target_area: TargetArea) -> Self {
    Self {
      target_area,
      attack_whitelist: None,
    }
  }

  pub fn limit_attacks(&mut self, enemy: &Enemy) -> Result<(), String> {
    if let Some(required_attack) = &enemy.required_attack {
      if let Some(attack_whitelist) = &mut self.attack_whitelist {
        //update allowed attacks
        *attack_whitelist = attack_whitelist.intersection(required_attack);
        if attack_whitelist.is_empty() {
          return Err("Attack whitelist is now empty".to_string());
        }
      } else {
        self.attack_whitelist = Some(required_attack.attacks());
      }
    }
    Ok(())
  }
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
  pub fn long(position: &Position) -> Self {
    Self::Long {
      column: position.column,
    }
  }

  pub fn wide(position: &Position, left_bound: bool) -> Self {
    Self::Wide {
      left_column: if left_bound {
        (position.column + Column.size() - 1) % Column.size()
      } else {
        position.column
      },
    }
  }

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
pub fn solve<'a, C>(
  arena: &SolvableArena,
  in_turns: Num,
  fast: bool,
  arena_solved_cache: C,
) -> Option<Vec<Move>>
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
      for amount in 1..=dimension.changes().size() {
        let move_ = Move::new(dimension, coordinate, amount, true).unwrap();
        let mut arena_clone = arena.clone();
        arena_clone.apply_move(move_);

        if let Some(mut solution) = solve(&arena_clone, in_turns - 1, fast, &mut *cache) {
          solution.insert(0, move_);

          if fast {
            return Some(solution);
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
  pub required_attack: Option<RequiredAttack>,
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

///A collection of [`Attack`]s an enemy can be damaged by
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Assoc)]
#[func(pub const fn symbol(& self) -> char)]
pub enum RequiredAttack {
  ///enemy must be attacked with a hammer or iron boots
  #[assoc(symbol = 'P')]
  IronBootsOrHammer,
  ///enemy must be attacked with a jump
  #[assoc(symbol = 'J')]
  Jump,
  ///enemy must be attacked with a hammer
  #[assoc(symbol = 'H')]
  Hammer,
}

impl RequiredAttack {
  pub fn attacks(&self) -> Vec<Attack> {
    match self {
      RequiredAttack::IronBootsOrHammer => vec![Attack::IronBoots, Attack::Hammer],
      RequiredAttack::Jump => vec![Attack::Jump, Attack::IronBoots],
      RequiredAttack::Hammer => vec![Attack::Hammer],
    }
  }
}

impl ToAttackVec for RequiredAttack {
  fn to_attack_vec(self) -> Vec<Attack> {
    self.attacks()
  }
}

impl ToAttackVec for &RequiredAttack {
  fn to_attack_vec(self) -> Vec<Attack> {
    self.attacks()
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Attack {
  Jump,
  Hammer,
  IronBoots,
}

impl ToAttackVec for Attack {
  fn to_attack_vec(self) -> Vec<Attack> {
    vec![self]
  }
}

pub trait ToAttackVec {
  fn to_attack_vec(self) -> Vec<Attack>;

  fn intersection<A>(self, other: A) -> Vec<Attack>
  where
    A: ToAttackVec,
    Self: Sized,
  {
    attack_intersection(self, other)
  }
}

impl<T> ToAttackVec for T
where
  T: AsRef<[Attack]>,
{
  fn to_attack_vec(self) -> Vec<Attack> {
    self.as_ref().to_vec()
  }
}

pub fn attack_intersection<A, B>(a: A, b: B) -> Vec<Attack>
where
  A: ToAttackVec,
  B: ToAttackVec,
{
  let b = b.to_attack_vec();
  a.to_attack_vec()
    .into_iter()
    .filter(|attack| b.contains(attack))
    .collect()
}

impl ToArenaSymbol for Enemy {
  fn to_arena_symbol(&self) -> char {
    if let Some(weakness) = &self.required_attack {
      weakness.symbol()
    } else {
      'E'
    }
  }
}

#[cfg(test)]
mod test_coverage {
  use crate::parse;
  use crate::solving::{AvailableEquipment, Coverage, SolvableArena};

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
      available_equipment: AvailableEquipment {
        throwing_hammer: false,
        ..Default::default()
      },
      ..Default::default()
    };
    for cmd in ["c4 1 H", "c4 23"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    Coverage::find(&arena)
      .ok_or(())
      .expect_err("no coverage should exists");
  }

  #[test]
  fn test_iron_boots_required() {
    let mut arena = SolvableArena::default();
    for cmd in [
      "c2 12", "c3 12", "c5 12", "c5 3 P", "c5 4 J", "c8 12", "c9 12",
    ] {
      parse(&mut arena, cmd).unwrap();
    }

    Coverage::find(&arena).expect("coverage should exist");
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

  fn steps<M>(moves: M) -> String
  where
    M: AsRef<[Move]>,
  {
    moves
      .as_ref()
      .iter()
      .map(|m| m.to_string())
      .collect::<Vec<_>>()
      .join(", ")
  }

  #[test]
  fn test_example_1() {
    let mut arena = SolvableArena::default();
    for cmd in ["c2 23", "c6 1234", "c8 14"] {
      parse(&mut arena, cmd).unwrap();
    }

    solve(&arena, 3, true, None).expect("is solvable in 3");
  }

  #[test]
  fn test_example_2() {
    let mut arena = SolvableArena::default();
    for cmd in [
      "c2 12", "c3 4", "c5 12", "c8 12", "c9 123", "c11 3", "c11 4",
    ] {
      parse(&mut arena, cmd).unwrap();
    }

    solve(&arena, 3, true, None).expect("is solvable in 3");
  }
}
