use std::any::type_name;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::io::{stdin, stdout, Write};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

use enum_assoc::Assoc;
use Dimension::{Column, Row};

pub type Num = u8;

fn main() {
  let mut arena = Arena::new();
  arena.show();
  loop {
    let mut line = String::new();

    print!("> ");
    stdout().flush().expect("failed to flush stdout");
    stdin()
      .read_line(&mut line)
      .expect("failed to read command line");
    if let Err(e) = parse(&mut arena, line.trim()) {
      eprintln!("{}", e);
    }
  }
}

pub fn parse(arena: &mut Arena, command: &str) -> Result<(), ParseError> {
  let mut args = command.split_whitespace().peekable();
  let cmd = args.next().unwrap();
  match cmd {
    "help" | "h" | "?" => {
      println!("set enemy positions: c1 124 H/J");
      println!("remove enemies: - c1 3");
      println!("set number of enemy groups: g 4");
      println!("solve: solve in 3");
      println!("whether you have a throw hammer: +hammer / -hammer");
      println!("manually execute turns: e r2 5");
      println!("clear arena: clear");
    }
    "clear" => {
      *arena = Arena::new();
      println!("arena has been cleared");
    }
    "g" | "groups" => {
      let arg = args
        .next()
        .ok_or(ParseError::missing_argument("number of groups"))?;
      let num_groups = arg
        .parse()
        .map_err(|e| ParseError::error(arg, "not a number", e))?;
      arena.num_groups = Some(num_groups);
      println!("set enemy groups to {}", num_groups);
    }
    "e" | "execute" | "run" => {
      let move_: Move = args
        .take(2)
        .collect::<Vec<_>>()
        .join(" ")
        .parse()
        .map_err(|e| ParseError::error("move", "invalid move", e))?;
      arena.apply_move(move_);
      arena.show();
    }
    "solve" => {
      let mut num_turns = None;
      let mut fast = false;
      if let Some(&"fast") = args.peek() {
        fast = true;
        args.next();
      }
      if let Some(arg) = args.next() {
        if arg != "in" {
          return Err(ParseError::illegal_argument(arg, "expected in"));
        }
        let arg = args
          .next()
          .ok_or(ParseError::missing_argument("number of turns"))?;
        let turns = arg
          .parse::<Num>()
          .map_err(|e| ParseError::error(arg, "not a number", e))?;
        num_turns = Some(turns);
      }

      println!("solving...");
      if let Some(in_turns) = num_turns {
        if let Some(solution) = solve(arena, in_turns, fast, None) {
          if solution.is_empty() {
            println!("Arena is already solved!");
          } else {
            println!(
              "Solution: {}",
              solution
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(", ")
            );
          }
        } else {
          println!("no solution was found :(");
        }
      } else {
        let mut cache = HashMap::new();
        for in_turns in 1..=100 {
          if let Some(solution) = solve(arena, in_turns, fast, &mut cache) {
            if solution.is_empty() {
              println!("Arena is already solved!");
            } else {
              println!(
                "solution was found in {} turns: {}",
                in_turns,
                solution
                  .iter()
                  .map(|m| m.to_string())
                  .collect::<Vec<_>>()
                  .join(", ")
              );
            }
            break;
          }
        }
      }
    }
    "-" | "undo" => {
      let column_arg = args.next().ok_or(ParseError::missing_argument("column"))?;
      let rows_arg = args.next().ok_or(ParseError::missing_argument("rows"))?;
      let positions = parse_positions(column_arg, rows_arg)?;
      for position in &positions {
        arena.remove(position);
      }
      arena.show();
    }
    "+hammer" => {
      arena.throwing_hammer_available = true;
    }
    "-hammer" => {
      arena.throwing_hammer_available = false;
    }
    _ => {
      let rows_arg = args.next().ok_or(ParseError::missing_argument("rows"))?;
      let weakness = match args.next() {
        Some(arg) => match arg {
          "H" => Attack::Hammer,
          "J" => Attack::Jump,
          _ => return Err(ParseError::illegal_argument(arg, "expected H or J")),
        }
        .into(),
        None => None,
      };
      let positions = parse_positions(cmd, rows_arg)?;
      for position in positions {
        arena.add(Enemy { position, weakness });
      }
      arena.show();
    }
  }
  Ok(())
}

fn parse_positions(column_arg: &str, rows_arg: &str) -> Result<Vec<Position>, ParseError> {
  if !column_arg.starts_with('c') {
    return Err(ParseError::unknown_command(column_arg));
  }

  let column_number_arg = &column_arg[1..];
  let column_number = column_number_arg
    .parse::<Num>()
    .map_err(|e| ParseError::error(column_arg, "invalid column number", e))?
    .saturating_sub(1);
  let column_number = Column
    .adapt(column_number)
    .map_err(|e| ParseError::error(column_arg, "out of bounds", e))?;

  let mut positions = vec![];

  let mut rows_code = rows_arg
    .parse::<u16>()
    .map_err(|e| ParseError::error(rows_arg, "rows have to be numbers", e))?;
  while rows_code > 0 {
    let row_number = (rows_code % 10).saturating_sub(1) as u8;
    let position = Position::at(row_number, column_number).map_err(|e| {
      ParseError::error(format!("{} {}", column_arg, rows_code), "out of bounds", e)
    })?;
    positions.push(position);
    rows_code /= 10;
  }
  Ok(positions)
}

#[derive(Debug)]
pub enum ParseError {
  MissingArgument { argument_name: String },
  UnknownCommand { command: String },
  IllegalArgument { argument: String, reason: String },
}

impl ParseError {
  pub fn missing_argument<S>(argument_name: S) -> Self
  where
    S: ToString,
  {
    Self::MissingArgument {
      argument_name: argument_name.to_string(),
    }
  }

  pub fn unknown_command<S>(command: S) -> Self
  where
    S: ToString,
  {
    Self::UnknownCommand {
      command: command.to_string(),
    }
  }

  pub fn illegal_argument<A, R>(argument: A, reason: R) -> Self
  where
    A: ToString,
    R: ToString,
  {
    Self::IllegalArgument {
      argument: argument.to_string(),
      reason: reason.to_string(),
    }
  }

  pub fn error<A, R, E>(argument: A, detail_message: R, error: E) -> Self
  where
    A: ToString,
    R: ToString,
    E: Error,
  {
    Self::illegal_argument(
      argument,
      format!("{}: {}", detail_message.to_string(), error),
    )
  }
}

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      ParseError::MissingArgument { argument_name } => {
        write!(f, "missing argument: {}", argument_name)
      }
      ParseError::UnknownCommand { command } => {
        write!(f, "unknown command: {}", command)
      }
      ParseError::IllegalArgument { argument, reason } => {
        write!(f, "Illegal argument '{}': {}", argument, reason)
      }
    }
  }
}

impl Error for ParseError {}

///An arena where [`Enemy`]s can stand
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Arena {
  pub enemies: Vec<Enemy>,
  pub num_groups: Option<Num>,
  pub throwing_hammer_available: bool,
}

impl Arena {
  pub fn new() -> Self {
    Self {
      enemies: Vec::with_capacity(16),
      num_groups: None,
      throwing_hammer_available: true,
    }
  }

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

  pub fn add(&mut self, enemy: Enemy) {
    if let Some(present) = self.get_at_mut(&enemy) {
      *present = enemy
    } else {
      self.enemies.push(enemy);
    }
  }

  pub fn remove(&mut self, at: &Position) {
    self.enemies.retain(|enemy| enemy.position != *at)
  }

  pub fn get_at(&self, at: &Position) -> Option<&Enemy> {
    self.enemies.iter().find(|enemy| enemy.position == *at)
  }

  pub fn get_at_mut(&mut self, at: &Position) -> Option<&mut Enemy> {
    self.enemies.iter_mut().find(|enemy| enemy.position == *at)
  }

  pub fn apply_move(&mut self, move_: Move) {
    for enemy in &mut self.enemies {
      enemy.apply_move(move_);
    }
  }

  pub fn is_solved(&self) -> bool {
    Coverage::find(self).is_some()
  }

  pub fn show(&self) {
    println!("{}", self)
  }
}

impl Default for Arena {
  fn default() -> Self {
    Self::new()
  }
}

impl Display for Arena {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let sym = |c, r| {
      if let Some(enemy) = self.get_at(&Position::at(r, c).expect("can not display")) {
        match enemy.weakness {
          Some(weakness) => match weakness {
            Attack::Hammer => "H",
            Attack::Jump => "J",
          },
          None => "E",
        }
      } else {
        "."
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

#[derive(Debug, Clone, Default)]
pub struct Coverage {
  areas: Vec<EnemyArea>,
}

impl Coverage {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn find(arena: &Arena) -> Option<Self> {
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
    arena: &Arena,
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

pub fn solve<'a, C>(arena: &Arena, in_turns: Num, fast: bool, arena_solved_cache: C) -> Option<Vec<Move>>
where
  C: Into<Option<&'a mut HashMap<Arena, bool>>>,
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
#[func(pub fn symbol(& self) -> & str)]
pub enum Attack {
  ///attack enemies with a hammer
  #[assoc(symbol = "H")]
  Hammer,
  ///attack enemies with a jump
  #[assoc(symbol = "J")]
  Jump,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
  row: Num,
  column: Num,
}

impl Position {
  pub fn at<N>(row: N, column: N) -> Result<Self, OutOfBoundsError<N, N::Error>>
  where
    N: TryInto<Num> + Copy,
  {
    Ok(Self {
      row: Row.adapt(row)?,
      column: Column.adapt(column)?,
    })
  }

  pub fn apply_move(&mut self, move_: Move) {
    match move_.dimension {
      Row => {
        if self.row != move_.coordinate {
          return;
        }

        let d_size = Column.size();
        let offset = if move_.in_positive_direction {
          move_.amount
        } else {
          d_size - move_.amount % d_size
        };
        self.column = (self.column + offset) % d_size;
      }
      Column => {
        let mut in_positive_direction = move_.in_positive_direction;
        if self.column == (move_.coordinate + Column.size() / 2) % Column.size() {
          in_positive_direction = !in_positive_direction;
        } else if self.column != move_.coordinate {
          return;
        }

        let d_size = Row.size();
        let dd_size = 2 * d_size;
        let offset = if in_positive_direction {
          move_.amount
        } else {
          dd_size - move_.amount % dd_size
        };
        let mirror_row = (self.row + offset) % dd_size;
        if mirror_row < d_size {
          self.row = mirror_row;
        } else {
          self.row = d_size * 2 - 1 - mirror_row;
          self.column = (self.column + Column.size() / 2) % Column.size();
        }
        self.row = mirror_row.min(d_size * 2 - 1 - mirror_row)
      }
    }
  }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub struct Move {
  pub dimension: Dimension,
  pub coordinate: Num,
  pub amount: Num,
  ///means clockwise or outward, depending on the dimension
  pub in_positive_direction: bool,
}

impl Move {
  pub fn new<C, A>(
    dimension: Dimension,
    coordinate: C,
    amount: A,
    in_positive_direction: bool,
  ) -> Result<Self, MoveCreationError<C, C::Error, A::Error>>
  where
    C: TryInto<Num> + Copy,
    A: TryInto<Num>,
  {
    Ok(Self {
      dimension,
      coordinate: dimension
        .adapt(coordinate)
        .map_err(|e| MoveCreationError::Coordinate(e))?,
      amount: amount
        .try_into()
        .map_err(|e| MoveCreationError::Amount(e))?,
      in_positive_direction,
    })
  }

  pub fn normalized(&self) -> Self {
    let mut norm = *self;
    if norm.dimension == Column && norm.coordinate > Column.size() / 2 {
      norm.coordinate -= Column.size() / 2;
      norm.in_positive_direction ^= true;
    }
    let c_size = norm.dimension.changes().size();
    norm.amount %= c_size;
    if norm.amount > c_size / 2 {
      norm.amount = c_size - norm.amount;
      norm.in_positive_direction ^= true;
    }
    norm
  }
}

impl FromStr for Move {
  type Err = MoveParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let args: [&str; 2] = s
      .split_whitespace()
      .collect::<Vec<_>>()
      .try_into()
      .map_err(|_| MoveParseError::new(s, MoveParseErrorDetails::InvalidFormat))?;
    let arg = args[0];
    let dimension = if arg.starts_with('r') {
      Row
    } else if arg.starts_with('c') {
      Column
    } else {
      return Err(MoveParseError::new(
        s,
        MoveParseErrorDetails::InvalidDimension,
      ));
    };
    let coordinate_arg = &arg[1..];
    let coordinate = coordinate_arg.parse::<Num>().map_err(|e| {
      MoveParseError::new(
        s,
        MoveParseErrorDetails::NotANumber {
          argument_name: "coordinate".to_string(),
          conversion_error: e,
        },
      )
    })?.saturating_sub(1);
    let coordinate = dimension
      .adapt(coordinate)
      .map_err(|e| MoveParseError::new(s, MoveParseErrorDetails::InvalidCoordinate(e)))?;
    let arg = args[1];
    let mut coordinate_arg = arg;
    let in_positive_direction = if coordinate_arg.starts_with('-') {
      coordinate_arg = &coordinate_arg[1..];
      false
    } else {
      true
    };
    let amount = coordinate_arg.parse::<Num>().map_err(|e| {
      MoveParseError::new(
        s,
        MoveParseErrorDetails::NotANumber {
          argument_name: "amount".to_string(),
          conversion_error: e,
        },
      )
    })?;
    Ok(Move {
      dimension,
      coordinate,
      amount,
      in_positive_direction,
    })
  }
}

impl Display for Move {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let normalized = self.normalized();
    write!(
      f,
      "{}{} {}{}",
      match normalized.dimension {
        Row => 'r',
        Column => 'c',
      },
      normalized.coordinate + 1,
      if normalized.in_positive_direction {
        ""
      } else {
        "-"
      },
      normalized.amount
    )
  }
}

#[derive(Debug)]
pub struct MoveParseError {
  pub value: String,
  pub details: MoveParseErrorDetails,
}

impl MoveParseError {
  pub fn new(value: &str, details: MoveParseErrorDetails) -> Self {
    Self {
      value: value.to_string(),
      details,
    }
  }
}

impl Display for MoveParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    let (description, details) = match &self.details {
      MoveParseErrorDetails::InvalidFormat => (
        "Invalid format".to_string(),
        "Needs to be '<r|c><coordinate> [-]<amount>'".to_string(),
      ),
      MoveParseErrorDetails::InvalidDimension => (
        "Invalid dimension identifier".to_string(),
        " Needs to be 'r' or 'c'".to_string(),
      ),
      MoveParseErrorDetails::NotANumber {
        argument_name,
        conversion_error,
      } => (
        format!("{} is not a number", argument_name),
        conversion_error.to_string(),
      ),
      MoveParseErrorDetails::InvalidCoordinate(e) => {
        ("Invalid coordinate".to_string(), e.to_string())
      }
    };
    write!(f, "{} for '{}': {}", description, self.value, details)
  }
}

impl Error for MoveParseError {}

#[derive(Debug)]
pub enum MoveParseErrorDetails {
  InvalidFormat,
  InvalidDimension,
  NotANumber {
    argument_name: String,
    conversion_error: <Num as FromStr>::Err,
  },
  InvalidCoordinate(OutOfBoundsError<Num, Infallible>),
}

#[derive(Debug)]
pub enum MoveCreationError<C, E, A> {
  Coordinate(OutOfBoundsError<C, E>),
  Amount(A),
}

impl<C, E, A> Display for MoveCreationError<C, E, A>
where
  C: Display,
  E: Display,
  A: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      MoveCreationError::Coordinate(e) => {
        write!(f, "invalid coordinate: {}", e)
      }
      MoveCreationError::Amount(e) => {
        write!(f, "invalid amount: {}", e)
      }
    }
  }
}

impl<C, E, A> Error for MoveCreationError<C, E, A>
where
  C: Display + Debug,
  E: Display + Debug,
  A: Display + Debug,
{
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Default, Assoc)]
#[func(pub fn size(& self) -> Num)]
#[func(pub fn name(& self) -> & str)]
#[func(pub fn changes(&self) -> Self)]
pub enum Dimension {
  #[default]
  #[assoc(size = 4)]
  #[assoc(name = "Row")]
  #[assoc(changes = Self::Column)]
  Row,
  #[assoc(size = 12)]
  #[assoc(name = "Column")]
  #[assoc(changes = Self::Row)]
  Column,
}

impl Dimension {
  pub fn adapt<N>(self, value: N) -> Result<Num, OutOfBoundsError<N, N::Error>>
  where
    N: TryInto<Num> + Copy,
  {
    let num = value.try_into().map_err(|e| OutOfBoundsError {
      dimension: self,
      value,
      conversion_error: Some(e),
    })?;
    if num >= self.size() {
      return Err(OutOfBoundsError {
        dimension: self,
        value,
        conversion_error: None,
      });
    }
    Ok(num)
  }

  ///gets the next coordinate in the positive direction
  pub fn next(&self, coordinate: Num) -> Num {
    (coordinate + 1) % self.size()
  }
}

impl Display for Dimension {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.name())
  }
}

#[derive(Debug)]
pub struct OutOfBoundsError<N, E> {
  pub dimension: Dimension,
  pub value: N,
  pub conversion_error: Option<E>,
}

impl<N, E> Display for OutOfBoundsError<N, E>
where
  N: Display,
  E: Display,
{
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    if let Some(conversion_error) = &self.conversion_error {
      write!(
        f,
        "Can't convert {} to {}: {}",
        self.value,
        type_name::<Num>(),
        conversion_error
      )
    } else {
      write!(
        f,
        "{} is too large for {} (0..{})",
        self.value,
        self.dimension,
        self.dimension.size()
      )
    }
  }
}

impl<N, E> Error for OutOfBoundsError<N, E>
where
  N: Display + Debug,
  E: Display + Debug,
{
}

#[cfg(test)]
mod test_dimension {
  use crate::Dimension;
  use crate::Dimension::Column;
  use Dimension::Row;

  #[test]
  fn test_next() {
    assert_eq!(1, Column.next(0));
    assert_eq!(2, Column.next(1));
    assert_eq!(0, Column.next(11));

    assert_eq!(1, Row.next(0));
    assert_eq!(2, Row.next(1));
    assert_eq!(0, Row.next(3));
  }
}

#[cfg(test)]
mod test_move {
  use crate::{Dimension, Move};

  #[test]
  fn test_parse() {
    assert_eq!(
      Move::new(Dimension::Column, 2, 1, false).unwrap(),
      "c3 -1".parse().expect("failed to parse")
    );
  }
}

#[cfg(test)]
mod test_position {
  use crate::Dimension::{Column, Row};
  use crate::{Move, Position};

  #[test]
  fn test_move_row() {
    let mut position = Position::at(2, 7).unwrap();
    let move_ = Move::new(Row, 2, 1, false).unwrap();
    position.apply_move(move_);

    assert_eq!(2, position.row);
    assert_eq!(6, position.column);
  }

  #[test]
  fn test_move_column_down() {
    let mut position = Position::at(0, 1).unwrap();
    let move_ = Move::new(Column, 1, 1, false).unwrap();
    position.apply_move(move_);

    assert_eq!(0, position.row);
    assert_eq!(7, position.column);
  }

  #[test]
  fn test_move_column_up() {
    let mut position = Position::at(0, 7).unwrap();
    let move_ = Move::new(Column, 1, 1, true).unwrap();
    position.apply_move(move_);

    assert_eq!(0, position.row);
    assert_eq!(1, position.column);
  }
}

#[cfg(test)]
mod test_coverage {
  use crate::{parse, Arena, Coverage};

  #[test]
  fn test_solved() {
    let mut arena = Arena::new();
    for cmd in ["c2 1234", "c4 12", "c5 12"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_some())
  }

  #[test]
  fn test_unsolved() {
    let mut arena = Arena::new();
    for cmd in ["c2 124", "c3 3", "c4 12", "c5 12"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_none())
  }

  #[test]
  fn test_too_few_enemies() {
    let mut arena = Arena::new();
    for cmd in ["c2 124", "c4 12", "c5 1"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_some())
  }

  #[test]
  fn test_ninja_covered() {
    let mut arena = Arena::new();
    parse(&mut arena, "c2 1234 J").expect("parse error");

    assert!(Coverage::find(&arena).is_some());
  }

  #[test]
  fn test_ninja_uncovered() {
    let mut arena = Arena::new();
    for cmd in ["c2 12 J", "c3 12 J"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    assert!(Coverage::find(&arena).is_none());
  }

  #[test]
  fn test_no_hammer() {
    let mut arena = Arena::new();
    arena.throwing_hammer_available = false;
    for cmd in ["c4 1 H", "c4 23"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    Coverage::find(&arena).ok_or(()).expect_err("no coverage should exists");
  }
}

#[cfg(test)]
mod test_solve {
  use crate::{parse, solve, Arena, Move};

  #[test]
  fn test_simple_solve() {
    let mut arena = Arena::new();
    for cmd in ["c2 124", "c3 3"] {
      parse(&mut arena, cmd).expect("parse error");
    }

    let solution = solve(&arena, 1, false, None).expect("is solvable");
    assert_eq!("r3 -1", steps(&solution));
  }

  #[test]
  fn test_two_steps() {
    let mut arena = Arena::new();
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
