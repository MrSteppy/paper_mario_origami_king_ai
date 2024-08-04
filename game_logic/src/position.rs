use std::any::type_name;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

use enum_assoc::Assoc;

use crate::position::Dimension::{Column, Row};

pub type Num = u8;

pub trait ToNum: TryInto<Num> + Copy {}

impl<T> ToNum for T where T: TryInto<Num> + Copy {}

pub type NumErr<N> = OutOfBoundsError<N, <N as TryInto<Num>>::Error>;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Position {
  pub row: Num,
  pub column: Num,
}

impl Position {
  pub fn at<N>(row: N, column: N) -> Result<Self, NumErr<N>>
  where
    N: ToNum,
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

  pub fn row(&self) -> Num {
    self.row
  }

  pub fn column(&self) -> Num {
    self.column
  }

  pub fn set_row<N>(&mut self, row: N) -> Result<(), NumErr<N>>
  where
    N: ToNum,
  {
    self.row = Row.adapt(row)?;
    Ok(())
  }

  pub fn set_column<N>(&mut self, column: N) -> Result<(), NumErr<N>>
  where
    N: ToNum,
  {
    self.column = Column.adapt(column)?;
    Ok(())
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
    C: ToNum,
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

  pub fn normalized(mut self) -> Self {
    match self.dimension {
      Row => {
        //turn by lowest amount possible
        self.amount %= Column.size();
        if self.amount > Column.size() / 2 {
          self.amount = Column.size() - self.amount;
          self.in_positive_direction ^= true; //invert
        }
      }
      Column => {
        //prefer lower coordinates
        if self.coordinate > Column.size() / 2 {
          self.coordinate -= Column.size() / 2;
          self.in_positive_direction ^= true; //invert
        }

        //prefer absolute smaller amount, then positive amount
        self.amount %= Row.size() * 2;
        if self.amount > Row.size() {
          self.amount = Row.size() * 2 - self.amount;
          self.in_positive_direction ^= true; //invert
        }
        if self.amount == Row.size() {
          self.in_positive_direction = true;
        }
      }
    }
    self
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
    let coordinate = coordinate_arg
      .parse::<Num>()
      .map_err(|e| {
        MoveParseError::new(
          s,
          MoveParseErrorDetails::NotANumber {
            argument_name: "coordinate".to_string(),
            conversion_error: e,
          },
        )
      })?
      .saturating_sub(1);
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
  pub fn adapt<N>(self, value: N) -> Result<Num, NumErr<N>>
  where
    N: ToNum,
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
  use crate::position::Dimension::Column;
  use crate::position::Dimension::Row;

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
  use std::str::FromStr;

  use crate::position::{Dimension, Move};

  #[test]
  fn test_parse() {
    assert_eq!(
      Move::new(Dimension::Column, 2, 1, false).unwrap(),
      "c3 -1".parse().expect("failed to parse")
    );
  }

  #[test]
  fn test_normalized() {
    assert_eq!(
      Move::from_str("r1 -3").unwrap(),
      Move::from_str("r1 9").unwrap().normalized()
    );
    assert_eq!(
      Move::from_str("c2 2").unwrap(),
      Move::from_str("c8 -2").unwrap().normalized()
    );
    assert_eq!(
      Move::from_str("c3 4").unwrap(),
      Move::from_str("c9 4").unwrap().normalized()
    );
  }

  #[test]
  fn test_display() {
    assert_eq!("c1 4", Move::from_str("c1 4").unwrap().to_string());
  }
}

#[cfg(test)]
mod test_position {
  use crate::position::{Move, Position};
  use crate::position::Dimension::{Column, Row};

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
