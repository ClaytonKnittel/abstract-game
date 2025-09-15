use std::{
  fmt::Display,
  io::{stdin, Stdout, Write},
};

use termion::{
  clear, cursor,
  screen::{AlternateScreen, IntoAlternateScreen},
};

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::player::Player,
  Game, GamePlayer, GameResult,
};

pub struct TermInterface<G, P1, P2> {
  game: G,
  player1: P1,
  player2: P2,
  stdout: AlternateScreen<Stdout>,
}

impl<G, P1, P2> TermInterface<G, P1, P2>
where
  G: Game + Display,
  P1: Player<Game = G>,
  P2: Player<Game = G>,
{
  pub fn new(game: G, player1: P1, player2: P2) -> GameInterfaceResult<Self> {
    let stdout = std::io::stdout().into_alternate_screen().map_err(|err| {
      GameInterfaceError::IoError(format!("Failed to enter alternate screen: {err}"))
    })?;
    Ok(Self { game, player1, player2, stdout })
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
        Err(GameInterfaceError::Quit | GameInterfaceError::IoError(_)) => break move_result,
        Err(err) => {
          self.println(&format!("{err}"))?;
        }
      }
    }
  }

  fn print(&mut self, str: &str) -> GameInterfaceResult {
    self
      .stdout
      .write_fmt(format_args!("{str}"))
      .map_err(|err| GameInterfaceError::IoError(format!("{err}")))
  }

  fn println(&mut self, str: &str) -> GameInterfaceResult {
    self
      .stdout
      .write_fmt(format_args!("{str}\n"))
      .map_err(|err| GameInterfaceError::IoError(format!("{err}")))
  }

  fn clear(&mut self) -> GameInterfaceResult {
    self.print(&format!("{}{}", cursor::Goto(1, 1), clear::All))
  }

  pub fn play(mut self) -> GameInterfaceResult {
    while !self.game.finished().is_finished() {
      self.println(&format!("{}", self.game))?;
      if let Some(flavor_text) = match self.game.current_player() {
        GamePlayer::Player1 => self.player1.prompt_move_text(&self.game),
        GamePlayer::Player2 => self.player2.prompt_move_text(&self.game),
      } {
        self.println(&flavor_text)?;
      } else {
        self.println(&format!("{} to move:", self.current_player_name()))?;
      }

      let m = self.next_move()?;
      self.game.make_move(m);

      self.clear()?;
    }

    self.println(&format!("{}", self.game))?;

    match self.game.finished() {
      GameResult::Win(player) => {
        self.println(&format!("{} wins!", self.player_name(player)))?;
      }
      GameResult::Tie => {
        self.println(&format!("It's a tie!"))?;
      }
      GameResult::NotFinished => unreachable!(),
    }

    // Wait for the user to press enter to end the program, so they may see the
    // result of the game.
    stdin()
      .read_line(&mut String::new())
      .map_err(|err| GameInterfaceError::IoError(err.to_string()))?;

    Ok(())
  }
}
