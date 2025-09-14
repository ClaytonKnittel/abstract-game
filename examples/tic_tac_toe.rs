use abstract_game::{
  human_players::tic_tac_toe_player::TicTacToePlayer,
  interactive::{human_term_player::HumanTermPlayer, term_interface::TermInterface},
  test_games::TicTacToe,
};

fn main() {
  let player1 = HumanTermPlayer::new("Player 1".to_owned(), TicTacToePlayer);
  let player2 = HumanTermPlayer::new("Player 2".to_owned(), TicTacToePlayer);
  let game = TicTacToe::new();

  let result = TermInterface::new(game, player1, player2).map(TermInterface::play);
  if let Err(err) = result {
    println!("{err}");
  }
}
