use std::fmt::{Debug, Display};

use crate::{Score, ScoreValue};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DeterminedScore {
  value: ScoreValue,
  moves_to_win: u32,
}

impl DeterminedScore {
  pub const fn tie() -> Self {
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

  pub fn from_score(score: Score) -> Option<Self> {
    score.fully_determined().then(|| {
      let depth = score.determined_depth();
      let value = score.score_at_depth(depth);
      debug_assert!(!value.is_tied() || score == Score::guaranteed_tie());

      let moves_to_win = if value.is_tied() { 0 } else { depth };
      Self { value, moves_to_win }
    })
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
      ScoreValue::Tie => write!(f, "[tie]"),
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
      some(eq(DeterminedScore::tie()))
    );

    expect_that!(DeterminedScore::from_score(Score::NO_INFO), none());
    expect_that!(DeterminedScore::from_score(Score::tie(1)), none());
    expect_that!(DeterminedScore::from_score(Score::tie(3)), none());

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
