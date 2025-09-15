use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::player::{MakeMoveControl, Player},
  Game, Solver,
};

pub struct BotPlayer<S> {
  name: String,
  solver: S,
  depth: u32,
}

impl<S> BotPlayer<S> {
  pub fn new(name: String, solver: S, depth: u32) -> Self {
    Self { name, solver, depth }
  }
}

impl<S: Solver> Player for BotPlayer<S> {
  type Game = S::Game;

  fn display_name(&self) -> String {
    self.name.clone()
  }

  fn make_move(
    &mut self,
    game: &S::Game,
  ) -> GameInterfaceResult<MakeMoveControl<<S::Game as Game>::Move>> {
    let (score, m) = self.solver.best_move(game, self.depth);
    let m = m.ok_or_else(|| {
      GameInterfaceError::InternalError(format!("No move found for game:\n{game:?}"))
    })?;

    eprintln!("Score {score} for game\n{game:?}");
    Ok(MakeMoveControl::Done(m))
  }
}
