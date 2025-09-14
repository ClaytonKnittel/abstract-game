use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum GameInterfaceError {
  Quit,
  MalformedMove(String),
  IoError(String),
}

impl Error for GameInterfaceError {}

impl Display for GameInterfaceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Quit => write!(f, "The user quit"),
      Self::MalformedMove(error) => write!(f, "Malformed move: {error}"),
      Self::IoError(error) => write!(f, "IO error: {error}"),
    }
  }
}

pub type GameInterfaceResult<T = ()> = Result<T, GameInterfaceError>;
