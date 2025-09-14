use crate::{error::GameInterfaceResult, Game};

pub trait Player {
  type Game: Game;

  fn display_name(&self) -> String;

  /// If `Some`, flavor text to print to the screen when prompting for the next
  /// move.
  fn prompt_move_text(&self, _game: &Self::Game) -> Option<String> {
    None
  }

  fn make_move(&mut self, game: &Self::Game) -> GameInterfaceResult<<Self::Game as Game>::Move>;
}
