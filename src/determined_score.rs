use std::fmt::{Debug, Display};

use crate::{Score, ScoreValue};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeterminedScore {
  value: ScoreValue,
  moves_to_win: u32,
}

impl DeterminedScore {
  pub const fn tie(depth: u32) -> Self {
    Self {
      value: ScoreValue::Tie,
      moves_to_win: depth,
    }
  }

  pub const fn guaranteed_tie() -> Self {
    Self { value: ScoreValue::Tie, moves_to_win: 0 }
  }

  pub const fn win(moves_to_win: u32) -> Self {
    Self {
      value: ScoreValue::CurrentPlayerWins,
      moves_to_win,
    }
  }

  pub const fn lose(moves_to_win: u32) -> Self {
    Self {
      value: ScoreValue::OtherPlayerWins,
      moves_to_win,
    }
  }

  /// Returns true if this score is a tie and is discovered to at least the
  /// given depth.
  pub fn is_tied_to(&self, depth: u32) -> bool {
    self.value == ScoreValue::Tie && (self.moves_to_win >= depth || self.moves_to_win == 0)
  }

  pub fn from_score(score: Score) -> Option<Self> {
    if score == Score::NO_INFO {
      None
    } else if score.is_guaranteed_tie() {
      Some(Self::guaranteed_tie())
    } else if score.is_tie() {
      Some(Self::tie(score.determined_depth()))
    } else {
      score.fully_determined().then(|| {
        let depth = score.determined_depth();
        let value = score.score_at_depth(depth);
        debug_assert!(!value.is_tied());

        Self { value, moves_to_win: depth }
      })
    }
  }
}

impl Debug for DeterminedScore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{self}")
  }
}

impl Display for DeterminedScore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.value {
      ScoreValue::CurrentPlayerWins => write!(f, "[cur:{}]", self.moves_to_win),
      ScoreValue::OtherPlayerWins => write!(f, "[oth:{}]", self.moves_to_win),
      ScoreValue::Tie => {
        if self.moves_to_win == 0 {
          write!(f, "[tie]")
        } else {
          write!(f, "[tie:{}]", self.moves_to_win)
        }
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use googletest::{gtest, prelude::*};

  use crate::{determined_score::DeterminedScore, Score};

  #[gtest]
  fn test_from_score() {
    expect_that!(
      DeterminedScore::from_score(Score::guaranteed_tie()),
      some(eq(DeterminedScore::guaranteed_tie()))
    );
    expect_that!(
      DeterminedScore::from_score(Score::tie(4)),
      some(eq(DeterminedScore::tie(4)))
    );

    expect_that!(DeterminedScore::from_score(Score::NO_INFO), none());

    expect_that!(
      DeterminedScore::from_score(Score::optimal_win(4)),
      some(eq(DeterminedScore::win(4)))
    );
    expect_that!(DeterminedScore::from_score(Score::win(5)), none());

    expect_that!(
      DeterminedScore::from_score(Score::optimal_lose(8)),
      some(eq(DeterminedScore::lose(8)))
    );
    expect_that!(DeterminedScore::from_score(Score::lose(6)), none());
  }
}
