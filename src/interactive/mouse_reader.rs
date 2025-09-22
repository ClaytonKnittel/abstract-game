use std::{io::Read, iter::FilterMap};

use termion::{
  event::{Event, Key, MouseButton, MouseEvent},
  input::{Events, TermRead},
};

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::input_reader::InputReader,
};

pub struct MouseButtonPress {
  pub button: MouseButton,
  pub x: u16,
  pub y: u16,
}

pub struct MouseReader<I> {
  input_stream: FilterMap<
    Events<I>,
    fn(
      event: Result<Event, std::io::Error>,
    ) -> Option<Result<MouseButtonPress, GameInterfaceError>>,
  >,
}

impl<I: Read + TermRead + 'static> MouseReader<I> {
  pub fn new(input: I) -> Self {
    Self {
      input_stream: input.events().filter_map(|event| match event {
        Ok(Event::Mouse(MouseEvent::Press(button, x, y))) => {
          Some(Ok(MouseButtonPress { button, x, y }))
        }
        Ok(Event::Key(Key::Char('q'))) => Some(Err(GameInterfaceError::Quit)),
        Ok(_) => None,
        Err(err) => Some(Err(GameInterfaceError::IoError(err.to_string()))),
      }),
    }
  }
}

impl<I: Read + TermRead> InputReader for MouseReader<I> {
  type Output = MouseButtonPress;

  /// Reads the next input from the input source, returning an error if the
  /// user quit or the underlying reader returned an error when trying to read
  /// the next line.
  fn next_input(&mut self) -> GameInterfaceResult<Self::Output> {
    self.input_stream.next().unwrap()
  }
}
