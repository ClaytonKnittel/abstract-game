use crate::{error::GameInterfaceResult, Game};

/// Value returned from `make_move` to tell the game engine whether to accept a
/// move from a player, or to keep prompting the player. `Continue` may be used
/// if a move requires multiple selections from the user.
pub enum MakeMoveControl<M> {
  /// The move that the player chose.
  Done(M),
  /// Continue prompting for a move. The internal state of the player should
  /// have updated to ask for different information.
  Continue,
}

pub trait Player {
  type Game: Game;

  fn display_name(&self) -> String;

  /// If `Some`, flavor text to print to the screen when prompting for the next
  /// move.
  fn prompt_move_text(&self, _game: &Self::Game) -> Option<String> {
    None
  }

  fn make_move(
    &mut self,
    game: &Self::Game,
  ) -> GameInterfaceResult<MakeMoveControl<<Self::Game as Game>::Move>>;
}
