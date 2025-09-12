use std::iter::successors;

use crate::{Game, GameResult, Score};

pub enum MoveLoss {
  Equivalent,
  Worse,
}

pub trait Solver {
  type Game: Game;

  fn best_move(
    &mut self,
    game: &Self::Game,
    depth: u32,
  ) -> (Score, Option<<Self::Game as Game>::Move>);

  fn move_loss(
    &mut self,
    m: <Self::Game as Game>::Move,
    game: &Self::Game,
    depth: u32,
  ) -> MoveLoss {
    debug_assert!(!game.finished().is_finished());
    let (cur_score, _) = self.best_move(game, depth);
    let (move_score, _) = self.best_move(&game.with_move(m), depth - 1);
    let move_score = move_score.backstep();

    if cur_score.compatible(move_score) {
      MoveLoss::Equivalent
    } else {
      debug_assert!(cur_score.better(move_score));
      MoveLoss::Worse
    }
  }

  fn playout(
    &mut self,
    game: &Self::Game,
    depth: u32,
  ) -> impl Iterator<Item = (Self::Game, <Self::Game as Game>::Move)> {
    let (_, m) = self.best_move(game, depth);
    successors(m.map(|m| (game.with_move(m), m)), move |(game, _)| {
      if matches!(game.finished(), GameResult::Win(_) | GameResult::Tie) {
        return None;
      }

      let (_, m) = self.best_move(game, depth);
      m.map(|m| (game.with_move(m), m))
    })
  }
}
