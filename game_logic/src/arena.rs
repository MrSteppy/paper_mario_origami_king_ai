use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::position::{Move, Position};

///An arena where something can stand
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Arena<E>
where
  E: Clone,
{
  pub enemies: Vec<E>,
}

impl<E> Arena<E>
where
  E: Clone + Deref<Target = Position>,
{
  pub fn add(&mut self, enemy: E) {
    if let Some(present) = self.get_at_mut(&enemy) {
      *present = enemy
    } else {
      self.enemies.push(enemy);
    }
  }

  pub fn remove(&mut self, at: &Position) {
    self.enemies.retain(|enemy| enemy.deref() != at)
  }

  pub fn get_at(&self, at: &Position) -> Option<&E> {
    self.enemies.iter().find(|&enemy| enemy.deref() == at)
  }

  pub fn get_at_mut(&mut self, at: &Position) -> Option<&mut E> {
    self
      .enemies
      .iter_mut()
      .find(|enemy| enemy.deref() as &Position == at)
  }

  pub fn show(&self)
  where
    E: ToArenaSymbol,
  {
    println!("{}", self)
  }
}

impl<E> Arena<E>
where
  E: Clone + DerefMut<Target = Position>,
{
  pub fn apply_move(&mut self, move_: Move) {
    for enemy in &mut self.enemies {
      enemy.apply_move(move_);
    }
  }
}

impl<E> Default for Arena<E>
where
  E: Clone,
{
  fn default() -> Self {
    Self {
      enemies: Vec::with_capacity(16),
    }
  }
}

impl<E> Display for Arena<E>
where
  E: Clone + Deref<Target = Position> + ToArenaSymbol,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let sym = |c, r| {
      if let Some(enemy) = self.get_at(&Position::at(r, c).expect("can not display")) {
        enemy.to_arena_symbol()
      } else {
        '.'
      }
    };
    writeln!(
      f,
      "  {}       {} {}       {}  ({} enemies)",
      sym(10, 3),
      sym(11, 3),
      sym(0, 3),
      sym(1, 3),
      self.enemies.len()
    )?;
    writeln!(
      f,
      "    {}     {} {}     {}  ",
      sym(10, 2),
      sym(11, 2),
      sym(0, 2),
      sym(1, 2)
    )?;
    writeln!(
      f,
      "      {}   {} {}   {}    ",
      sym(10, 1),
      sym(11, 1),
      sym(0, 1),
      sym(1, 1)
    )?;
    writeln!(
      f,
      "        {} {} {} {}      ",
      sym(10, 0),
      sym(11, 0),
      sym(0, 0),
      sym(1, 0)
    )?;
    writeln!(
      f,
      "{} {} {} {}         {} {} {} {}",
      sym(9, 3),
      sym(9, 2),
      sym(9, 1),
      sym(9, 0),
      sym(2, 0),
      sym(2, 1),
      sym(2, 2),
      sym(2, 3)
    )?;
    writeln!(
      f,
      "{} {} {} {}         {} {} {} {}",
      sym(8, 3),
      sym(8, 2),
      sym(8, 1),
      sym(8, 0),
      sym(3, 0),
      sym(3, 1),
      sym(3, 2),
      sym(3, 3)
    )?;
    writeln!(
      f,
      "        {} {} {} {}      ",
      sym(7, 0),
      sym(6, 0),
      sym(5, 0),
      sym(4, 0)
    )?;
    writeln!(
      f,
      "      {}   {} {}   {}    ",
      sym(7, 1),
      sym(6, 1),
      sym(5, 1),
      sym(4, 1)
    )?;
    writeln!(
      f,
      "    {}     {} {}     {}  ",
      sym(7, 2),
      sym(6, 2),
      sym(5, 2),
      sym(4, 2)
    )?;
    write!(
      f,
      "  {}       {} {}       {}",
      sym(7, 3),
      sym(6, 3),
      sym(5, 3),
      sym(4, 3)
    )
  }
}

pub trait ToArenaSymbol {
  fn to_arena_symbol(&self) -> char;
}
