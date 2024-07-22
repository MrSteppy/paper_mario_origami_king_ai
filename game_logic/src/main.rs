use std::fmt::{Debug, Display};
use std::io::{stdin, stdout, Write};

use game_logic::solving::SolvableArena;

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
    if let Err(e) = game_logic::parse(&mut arena, line.trim()) {
      eprintln!("{}", e);
    }
  }
}

