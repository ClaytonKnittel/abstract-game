use std::io::stdin;

use itertools::Itertools;

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::{human_player::HumanPlayer, player::Player},
  Game,
};

pub struct HumanTermPlayer<P> {
  name: String,
  player: P,
}

impl<P> HumanTermPlayer<P> {
  pub fn new(name: String, player: P) -> Self {
    Self { name, player }
  }
}

impl<P: HumanPlayer> Player for HumanTermPlayer<P> {
  type Game = P::Game;

  fn display_name(&self) -> String {
    self.name.clone()
  }

  fn prompt_move_text(&self, game: &Self::Game) -> Option<String> {
    Some(self.player.prompt_move_text(game))
  }

  fn make_move(&mut self, game: &Self::Game) -> GameInterfaceResult<<P::Game as Game>::Move> {
    let mut buffer = String::new();
    stdin()
      .read_line(&mut buffer)
      .map_err(|err| GameInterfaceError::IoError(err.to_string()))?;
    let move_text = buffer.trim();
    if move_text == "q" {
      return Err(GameInterfaceError::Quit);
    }

    let m = self.player.parse_move(move_text, game)?;

    if !game.each_move().contains(&m) {
      return Err(GameInterfaceError::MalformedMove(format!(
        "{m:?} is not a legal move!"
      )));
    }

    Ok(m)
  }
}
