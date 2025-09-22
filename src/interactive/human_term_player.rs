use std::io::{stdin, BufReader};

use itertools::Itertools;

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::{
    human_player::HumanPlayer,
    input_reader::InputReader,
    line_reader::GameMoveLineReader,
    player::{MakeMoveControl, Player},
  },
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

  fn make_move<I: InputReader>(
    &mut self,
    game: &Self::Game,
    input: &mut I,
  ) -> GameInterfaceResult<MakeMoveControl<<P::Game as Game>::Move>> {
    let m = self.player.parse_move(input, game)?;

    if let MakeMoveControl::Done(m) = &m {
      if !game.each_move().contains(m) {
        return Err(GameInterfaceError::MalformedMove(format!(
          "{m:?} is not a legal move!"
        )));
      }
    }

    Ok(m)
  }
}
