use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::io::{stdin, stdout, Write};

use position::{Move, Num, Position};
use position::Dimension::Column;
use solving::{Attack, Enemy};

use crate::solving::SolvableArena;

mod arena;
mod position;
mod solving;

fn main() {
  let mut arena = SolvableArena::default();
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

pub fn parse(arena: &mut SolvableArena, command: &str) -> Result<(), ParseError> {
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
      *arena = SolvableArena::default();
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
        if let Some(solution) = solving::solve(arena, in_turns, fast, None) {
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
          if let Some(solution) = solving::solve(arena, in_turns, fast, &mut cache) {
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

