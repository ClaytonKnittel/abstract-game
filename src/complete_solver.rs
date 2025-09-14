use crate::{determined_score::DeterminedScore, Game, Solver};

/// Complete solvers find the true optimal moves (e.g. highest-valued `Score`),
/// which differs from "optimal" solvers (e.g. "never loses") in that the
/// minimum path to victory is required.
///
/// You should only implement this trait if you know that your solver is a
/// complete solver.
pub trait CompleteSolver: Solver {
  fn best_move_determined(
    &mut self,
    game: &Self::Game,
    depth: u32,
  ) -> (DeterminedScore, Option<<Self::Game as Game>::Move>) {
    let (score, m) = Solver::best_move(self, game, depth);
    let score = DeterminedScore::from_score(score)
      .expect(&format!("Expected a determined score, got {score}"));
    (score, m)
  }
}
