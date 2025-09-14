use crate::{
  error::{GameInterfaceError, GameInterfaceResult},
  interactive::player::Player,
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

  fn make_move(&mut self, game: &S::Game) -> GameInterfaceResult<<S::Game as Game>::Move> {
    self.solver.best_move(game, self.depth).1.ok_or_else(|| {
      GameInterfaceError::InternalError(format!("No move found for game:\n{game:?}"))
    })
  }
}
