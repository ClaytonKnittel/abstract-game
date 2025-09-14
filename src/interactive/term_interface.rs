use std::fmt::Display;

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::player::Player,
  Game, GamePlayer, GameResult,
};

pub struct TermInterface<G, P1, P2> {
  game: G,
  player1: P1,
  player2: P2,
}

impl<G, P1, P2> TermInterface<G, P1, P2>
where
  G: Game + Display,
  P1: Player<Game = G>,
  P2: Player<Game = G>,
{
  pub fn new(game: G, player1: P1, player2: P2) -> Self {
    Self { game, player1, player2 }
  }

  fn player_name(&self, player: GamePlayer) -> String {
    match player {
      GamePlayer::Player1 => self.player1.display_name(),
      GamePlayer::Player2 => self.player2.display_name(),
    }
  }

  fn current_player_name(&self) -> String {
    self.player_name(self.game.current_player())
  }

  fn next_move(&mut self) -> GameInterfaceResult<G::Move> {
    loop {
      let move_result = match self.game.current_player() {
        GamePlayer::Player1 => self.player1.make_move(&self.game),
        GamePlayer::Player2 => self.player2.make_move(&self.game),
      };

      match move_result {
        Ok(m) => break Ok(m),
        Err(GameInterfaceError::Quit) => break Err(GameInterfaceError::Quit),
        Err(err) => {
          println!("{err}");
        }
      }
    }
  }

  pub fn play(mut self) -> GameInterfaceResult {
    while !self.game.finished().is_finished() {
      println!("{}", self.game);
      println!("{} to move:", self.current_player_name());

      let m = self.next_move();
      self.game.make_move(m?);
    }

    match self.game.finished() {
      GameResult::Win(player) => {
        println!("{} wins!", self.player_name(player));
      }
      GameResult::Tie => {
        println!("It's a tie!");
      }
      GameResult::NotFinished => unreachable!(),
    }

    Ok(())
  }
}
