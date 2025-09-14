use crate::{error::GameInterfaceResult, Game};

pub trait Player {
  type Game: Game;

  fn display_name(&self) -> String;

  fn make_move(&mut self, game: &Self::Game) -> GameInterfaceResult<<Self::Game as Game>::Move>;
}
