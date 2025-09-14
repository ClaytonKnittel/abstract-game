use std::fmt::Display;

use crate::{Game, GameMoveIterator, GamePlayer, GameResult};

pub struct NimMoveIter {
  sticks: u32,
}

impl GameMoveIterator for NimMoveIter {
  type Game = Nim;

  fn next(&mut self, nim: &Nim) -> Option<u32> {
    if self.sticks >= Nim::MAX_STICKS_PER_TURN.min(nim.sticks) {
      None
    } else {
      self.sticks += 1;
      Some(self.sticks)
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Nim {
  sticks: u32,
  player1: bool,
}

impl Nim {
  pub const MAX_STICKS_PER_TURN: u32 = 2;

  pub fn new(sticks: u32) -> Self {
    Self { sticks, player1: true }
  }

  pub fn sticks(&self) -> u32 {
    self.sticks
  }
}

impl Game for Nim {
  type Move = u32;
  type MoveGenerator = NimMoveIter;

  fn move_generator(&self) -> NimMoveIter {
    NimMoveIter { sticks: 0 }
  }

  fn make_move(&mut self, sticks: u32) {
    debug_assert!(sticks <= self.sticks);
    self.sticks -= sticks;
    self.player1 = !self.player1;
  }

  fn current_player(&self) -> GamePlayer {
    if self.player1 {
      GamePlayer::Player1
    } else {
      GamePlayer::Player2
    }
  }

  fn finished(&self) -> GameResult {
    if self.sticks == 0 {
      GameResult::Win(if self.player1 {
        GamePlayer::Player2
      } else {
        GamePlayer::Player1
      })
    } else {
      GameResult::NotFinished
    }
  }
}

impl Display for Nim {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Sticks left: {}", self.sticks)
  }
}
