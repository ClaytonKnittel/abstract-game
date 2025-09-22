use std::{
  fmt::Display,
  io::{stdin, Stdin, Stdout, Write},
};

use termion::{
  clear, cursor,
  input::MouseTerminal,
  screen::{AlternateScreen, IntoAlternateScreen},
};

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::{
    line_reader::GameMoveLineReader,
    player::{MakeMoveControl, Player},
  },
  Game, GamePlayer, GameResult,
};

pub struct TermInterface<G, P1, P2, O, I> {
  game: G,
  player1: P1,
  player2: P2,
  stdout: O,
  stdin: I,
}

impl<G, P1, P2, O, I> TermInterface<G, P1, P2, O, I>
where
  G: Game + Display,
  P1: Player<Game = G>,
  P2: Player<Game = G>,
  O: Write,
{
  fn player_name(&self, player: GamePlayer) -> String {
    match player {
      GamePlayer::Player1 => self.player1.display_name(),
      GamePlayer::Player2 => self.player2.display_name(),
    }
  }

  fn current_player_name(&self) -> String {
    self.player_name(self.game.current_player())
  }

  fn next_move(&mut self) -> GameInterfaceResult<MakeMoveControl<G::Move>> {
    loop {
      let move_result = match self.game.current_player() {
        GamePlayer::Player1 => self.player1.make_move(&self.game),
        GamePlayer::Player2 => self.player2.make_move(&self.game),
      };

      match move_result {
        Ok(m) => break Ok(m),
        Err(err @ (GameInterfaceError::Quit | GameInterfaceError::IoError(_))) => break Err(err),
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

      // Prompt the player for their next move.
      let next_move = self.next_move()?;

      // Clear the screen before interpreting their move.
      self.clear()?;

      // If the player requested to continue, loop back and redraw the screen.
      // Otherwise, make the move and loop back.
      match next_move {
        MakeMoveControl::Done(m) => self.game.make_move(m),
        MakeMoveControl::Continue => continue,
      };
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

impl<G, P1, P2>
  TermInterface<G, P1, P2, MouseTerminal<AlternateScreen<Stdout>>, GameMoveLineReader<Stdin>>
where
  G: Game + Display,
  P1: Player<Game = G>,
  P2: Player<Game = G>,
{
  pub fn new(game: G, player1: P1, player2: P2) -> GameInterfaceResult<Self> {
    let stdout = MouseTerminal::from(
      std::io::stdout()
        // .into_raw_mode()
        // .map_err(|err| GameInterfaceError::IoError(format!("Failed to enter raw mode: {err}")))?
        .into_alternate_screen()
        .map_err(|err| {
          GameInterfaceError::IoError(format!("Failed to enter alternate screen: {err}"))
        })?,
    );
    let stdin = GameMoveLineReader { input: stdin() };
    Ok(Self { game, player1, player2, stdout, stdin })
  }
}
