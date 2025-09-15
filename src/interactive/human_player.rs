use std::io::BufRead;

use crate::{error::GameInterfaceResult, interactive::line_reader::GameMoveLineReader, Game};

pub trait HumanPlayer {
  type Game: Game;

  /// The text to be printed to the terminal when it's a player's turn to make
  /// a move.
  fn prompt_move_text(&self, game: &Self::Game) -> String;

  /// Parses a player's move, returning the parsed move, or an error if parsing
  /// failed.
  fn parse_move<I: BufRead>(
    &self,
    move_reader: GameMoveLineReader<I>,
    game: &Self::Game,
  ) -> GameInterfaceResult<<Self::Game as Game>::Move>;
}
