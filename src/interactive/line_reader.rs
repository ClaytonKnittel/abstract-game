use std::io::BufRead;

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::input_reader::InputReader,
};

pub struct GameMoveLineReader<I> {
  pub(crate) input: I,
}

impl<I: BufRead> InputReader for GameMoveLineReader<I> {
  type Output = String;

  /// Reads the next line from the input source, returning an error if the user
  /// quit or the underlying `BufReader` returned an error when trying to read
  /// the next line.
  fn next_input(&mut self) -> GameInterfaceResult<String> {
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
