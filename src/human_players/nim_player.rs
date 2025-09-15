use std::io::BufRead;

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::{human_player::HumanPlayer, line_reader::GameMoveLineReader},
  test_games::Nim,
};

pub struct NimPlayer;

impl HumanPlayer for NimPlayer {
  type Game = Nim;

  fn prompt_move_text(&self, game: &Nim) -> String {
    format!(
      "How many sticks would you like to take? {}",
      if game.sticks() == 1 {
        "1 is the only option"
      } else {
        "1 or 2"
      }
    )
  }

  fn parse_move<I: BufRead>(
    &self,
    mut move_reader: GameMoveLineReader<I>,
    game: &Nim,
  ) -> GameInterfaceResult<u32> {
    let move_text = move_reader.next_line()?;
    let sticks = move_text
      .parse()
      .map_err(|_| GameInterfaceError::MalformedMove(format!("{move_text} is not a number")))?;

    if sticks == 0 {
      return Err(GameInterfaceError::MalformedMove(
        "Can't take 0 sticks!".to_owned(),
      ));
    }
    if sticks > game.sticks().min(2) {
      return Err(GameInterfaceError::MalformedMove(format!(
        "{sticks} is greater than the number of sticks remaining ({})",
        game.sticks()
      )));
    }

    Ok(sticks)
  }
}
