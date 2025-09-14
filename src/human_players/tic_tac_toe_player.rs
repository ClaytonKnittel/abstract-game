use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::human_player::HumanPlayer,
  test_games::{TTTMove, TicTacToe},
  Game, GamePlayer,
};

pub struct TicTacToePlayer;

impl HumanPlayer for TicTacToePlayer {
  type Game = TicTacToe;

  fn prompt_move_text(&self, game: &TicTacToe) -> String {
    format!(
      "Where would you like to place the next {}?",
      match game.current_player() {
        GamePlayer::Player1 => 'X',
        GamePlayer::Player2 => 'O',
      }
    )
  }

  fn parse_move(&self, move_text: &str, game: &TicTacToe) -> GameInterfaceResult<TTTMove> {
    let make_malformed_move_err = || {
      GameInterfaceError::MalformedMove(format!(
        "\"{move_text}\" is not a valid coordinate pair \"X,Y\""
      ))
    };

    let mut chars = move_text.chars();
    let c1 = chars.next().ok_or_else(make_malformed_move_err)?;
    let c2 = chars.next().ok_or_else(make_malformed_move_err)?;
    let c3 = chars.next().ok_or_else(make_malformed_move_err)?;
    if chars.next().is_some() {
      return Err(GameInterfaceError::MalformedMove(format!(
        "Move string is greater than 3 characters long"
      )));
    }

    if c2 != ',' {
      return Err(GameInterfaceError::MalformedMove(format!(
        "Expected ',' in second position of move string"
      )));
    }

    if !('1'..='3').contains(&c1) {
      return Err(GameInterfaceError::MalformedMove(format!(
        "Expected a number from '1' - '3' as the x-coordinate, found {c1}"
      )));
    }
    if !('1'..='3').contains(&c3) {
      return Err(GameInterfaceError::MalformedMove(format!(
        "Expected a number from '1' - '3' as the y-coordinate, found {c3}"
      )));
    }
    let x = c1 as u32 - '1' as u32;
    let y = c3 as u32 - '1' as u32;

    if !game.is_empty((x, y)) {
      return Err(GameInterfaceError::MalformedMove(format!(
        "Tile ({x}, {y}) is already occupied!"
      )));
    }

    Ok(TTTMove::new((x, y)))
  }
}
