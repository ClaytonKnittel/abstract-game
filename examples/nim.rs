use abstract_game::{
  human_players::nim_player::NimPlayer,
  interactive::{human_term_player::HumanTermPlayer, term_interface::TermInterface},
  test_games::Nim,
};

fn main() {
  let player1 = HumanTermPlayer::new("Player 1".to_owned(), NimPlayer);
  let player2 = HumanTermPlayer::new("Player 2".to_owned(), NimPlayer);
  let game = Nim::new(20);

  let result = TermInterface::new(game, player1, player2).map(TermInterface::play);
  if let Err(err) = result {
    println!("{err}");
  }
}
