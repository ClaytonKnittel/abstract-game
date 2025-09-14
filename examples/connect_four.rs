use abstract_game::{
  human_players::connect_n_player::ConnectNPlayer,
  interactive::{human_term_player::HumanTermPlayer, term_interface::TermInterface},
  test_games::ConnectN,
};

fn main() {
  let player1 = HumanTermPlayer::new("Player 1".to_owned(), ConnectNPlayer);
  let player2 = HumanTermPlayer::new("Player 2".to_owned(), ConnectNPlayer);
  let game = ConnectN::new(7, 6, 4);

  let result = TermInterface::new(game, player1, player2).map(TermInterface::play);
  if let Err(err) = result {
    println!("{err}");
  }
}
