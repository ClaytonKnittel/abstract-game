use std::io::BufRead;

use itertools::Itertools;

use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::{
    human_player::HumanPlayer, input_reader::InputReader, line_reader::GameMoveLineReader,
    player::MakeMoveControl,
  },
  test_games::{ConnectMove, ConnectN},
  Game, GamePlayer,
};

pub struct ConnectNPlayer;

impl HumanPlayer for ConnectNPlayer {
  type Game = ConnectN;

  fn prompt_move_text(&self, game: &ConnectN) -> String {
    format!(
      "{}\n(Column index)\n\nPlayer {} turn (enter the column you'd like to play in):",
      (0..game.width()).map(|col| col.to_string()).join(" "),
      match game.current_player() {
        GamePlayer::Player1 => 'X',
        GamePlayer::Player2 => 'O',
      }
    )
  }

  fn parse_move<I: BufRead>(
    &self,
    mut move_reader: GameMoveLineReader<I>,
    _game: &ConnectN,
  ) -> GameInterfaceResult<MakeMoveControl<ConnectMove>> {
    let move_text = move_reader.next_input()?;
    let col = move_text
      .parse()
      .map_err(|_| GameInterfaceError::MalformedMove(format!("{move_text} is not a number.")))?;
    Ok(MakeMoveControl::Done(ConnectMove { col }))
  }
}
