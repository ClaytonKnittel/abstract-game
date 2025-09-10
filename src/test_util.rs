use itertools::Itertools;
use rand::Rng;

use crate::Game;

pub type AbstractGameResult<T = ()> = Result<T, String>;

pub fn make_deterministic_random_move<G: Game, R: Rng>(game: &mut G, rng: &mut R) -> Option<G::Move>
where
  G::Move: Ord,
{
  let mut moves = game.each_move().collect_vec();
  if moves.is_empty() {
    return None;
  }

  moves.sort();
  let m = moves[rng.random_range(0..moves.len())];
  game.make_move(m);
  Some(m)
}

/// Plays a random number of moves in the game, returning the number of moves
/// played until the game finished. If the game did not finish, returns
/// `num_moves + 1`.
pub fn deterministic_random_playout<G: Game, R: Rng>(
  game: &mut G,
  num_moves: usize,
  rng: &mut R,
) -> usize
where
  G::Move: Ord,
{
  for i in 1..=num_moves {
    make_deterministic_random_move(game, rng);
    if game.finished().is_finished() {
      return i;
    }
  }

  num_moves + 1
}

pub fn deterministic_random_unfinished_state<G: Game, R: Rng>(
  game: &G,
  num_moves: usize,
  rng: &mut R,
) -> AbstractGameResult<G>
where
  G::Move: Ord,
{
  const ATTEMPTS: u32 = 500;
  for _ in 0..ATTEMPTS {
    let mut tmp = game.clone();
    if deterministic_random_playout(&mut tmp, num_moves, rng) > num_moves {
      return Ok(tmp);
    }
  }

  Err(format!(
    "Failed to make an unfinished game after {ATTEMPTS} attempts"
  ))
}

pub fn generate_deterministic_random_unfinished_states<G: Game, R: Rng>(
  initial_state: &G,
  count: usize,
  num_moves: usize,
  rng: &mut R,
) -> AbstractGameResult<Vec<G>>
where
  G::Move: Ord,
{
  let mut states = Vec::with_capacity(count);

  let attempts = 100 * count;
  for _ in 0..attempts {
    let mut game = initial_state.clone();
    if deterministic_random_playout(&mut game, num_moves, rng) > num_moves {
      states.push(game);
    }
    if states.len() == count {
      return Ok(states);
    }
  }

  Err(format!(
    "Failed to generate {count} random states with {num_moves} moves after {attempts} attempts"
  ))
}

pub fn generate_deterministic_random_walks<G: Game, R: Rng>(
  initial_state: &G,
  count: usize,
  rng: &mut R,
) -> AbstractGameResult<Vec<Vec<G::Move>>>
where
  G::Move: Ord,
{
  const MAX_MOVES: usize = 1000;
  debug_assert!(!initial_state.finished().is_finished());

  (0..count)
    .map(|_| {
      let mut game = initial_state.clone();
      let mut moves = Vec::new();
      for _ in 0..MAX_MOVES {
        if game.finished().is_finished() {
          return Ok(moves);
        }
        if let Some(m) = make_deterministic_random_move(&mut game, rng) {
          moves.push(m);
        } else {
          return Ok(moves);
        }
      }

      Err(format!(
        "Exceeded maximum number of moves {MAX_MOVES} without finishing the game."
      ))
    })
    .collect()
}
