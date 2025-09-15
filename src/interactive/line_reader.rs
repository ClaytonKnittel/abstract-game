use std::io::BufRead;

use crate::error::{GameInterfaceError, GameInterfaceResult};

pub struct GameMoveLineReader<I> {
  pub(crate) input: I,
}

impl<I: BufRead> GameMoveLineReader<I> {
  pub fn next_line(&mut self) -> GameInterfaceResult<String> {
    let mut buffer = String::new();
    self
      .input
      .read_line(&mut buffer)
      .map_err(|err| GameInterfaceError::IoError(err.to_string()))?;

    let move_text = buffer.trim();
    if move_text == "q" {
      return Err(GameInterfaceError::Quit);
    }

    Ok(move_text.to_owned())
  }
}
