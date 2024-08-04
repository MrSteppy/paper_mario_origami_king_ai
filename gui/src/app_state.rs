use std::ops::{Deref, DerefMut};

use game_logic::arena::Arena;
use game_logic::position::{Dimension, Move, Num, Position};
use game_logic::solving::SolvableArena;

///Holds the current data of the app which should outlive different render and animation states
#[derive(Debug)]
pub struct AppState {
  pub arena: SolvableArena,
  pub arena_ground: Arena<Tile>,
  pub number_of_turns: Num,
  pub current_solution: Option<Solution>,
  pub height: i32, //temporary used while developing this app
}

impl Default for AppState {
  fn default() -> Self {
    let mut arena_ground = Arena::default();
    for column in 0..Dimension::Column.size() {
      for row in 0..Dimension::Row.size() {
        arena_ground.add(Tile {
          position: Position::at(row, column).unwrap(),
          color: if (column % 2 == 0) ^ (row % 2 == 0) {
            TileColor::Light
          } else {
            TileColor::Dark
          },
        })
      }
    }
    Self {
      arena: Default::default(),
      arena_ground,
      number_of_turns: 2,
      current_solution: None,
      height: 0,
    }
  }
}

impl AppState {
  pub fn apply_move(&mut self, move_: Move) {
    self.arena.apply_move(move_);
    self.arena_ground.apply_move(move_);
  }
}

#[derive(Debug)]
pub struct Solution {
  pub moves: Vec<Move>,
  pub executed_moves: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum TileColor {
  Light,
  Dark,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Tile {
  pub position: Position,
  pub color: TileColor,
}

impl Deref for Tile {
  type Target = Position;

  fn deref(&self) -> &Self::Target {
    &self.position
  }
}

impl DerefMut for Tile {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.position
  }
}
