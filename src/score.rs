use std::{
  cmp::Ordering,
  fmt::{Debug, Display},
  hint::unreachable_unchecked,
};

use crate::util::{max_u32, min_u32};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreValue {
  CurrentPlayerWins,
  OtherPlayerWins,
  Tie,
}

#[derive(Clone, Copy)]
pub struct Score {
  /// Layout:
  ///          31         30 -  23  22     -    12   11     -     0
  /// +------------------+--------+----------------+----------------+
  /// | cur player wins? | unused | turn count win | turn count tie |
  /// +------------------+--------+----------------+----------------+
  pub(crate) data: u32,
}

impl Score {
  const TIE_BITS: u32 = 12;
  const TIE_SHIFT: u32 = 0;
  const MAX_TIE_DEPTH: u32 = (1 << Self::TIE_BITS) - 1;
  const TIE_MASK: u32 = Self::MAX_TIE_DEPTH << Self::TIE_SHIFT;

  const WIN_BITS: u32 = 11;
  const WIN_SHIFT: u32 = Self::TIE_SHIFT + Self::TIE_BITS;
  const MAX_WIN_DEPTH: u32 = (1 << Self::WIN_BITS) - 1;
  const WIN_MASK: u32 = Self::MAX_WIN_DEPTH << Self::WIN_SHIFT;

  const UNUSED_BITS: u32 = 8;
  const UNUSED_SHIFT: u32 = Self::WIN_SHIFT + Self::WIN_BITS;

  const CUR_PLAYER_WINS_SHIFT: u32 = Self::UNUSED_SHIFT + Self::UNUSED_BITS;
  const CUR_PLAYER_WINS_MASK: u32 = 1 << Self::CUR_PLAYER_WINS_SHIFT;

  /// Mark the current player as winning with turn_count_win_ = 0, which is an
  /// impossible state to be in.
  const ANCESTOR: Score = Score::new(true, 0, 0);

  pub const fn new(cur_player_wins: bool, turn_count_tie: u32, turn_count_win: u32) -> Self {
    debug_assert!(turn_count_tie <= Self::MAX_TIE_DEPTH);
    debug_assert!(turn_count_win <= Self::MAX_WIN_DEPTH);
    Self {
      data: Self::pack(cur_player_wins, turn_count_tie, turn_count_win),
    }
  }

  /// Construct a `Score` that contains no information.
  pub const fn no_info() -> Self {
    Self::tie(0)
  }

  /// Returns true if this score contains no info.
  pub const fn has_no_info(&self) -> bool {
    !self.cur_player_wins() && self.turn_count_tie() == 0 && self.turn_count_win() == 0
  }

  pub const fn is_tie(&self) -> bool {
    self.turn_count_win() == 0
  }

  /// Returns true if this score represents an ancestor, e.g. is currently being computed.
  pub const fn is_ancestor(&self) -> bool {
    self.data == Self::ANCESTOR.data
  }

  /// Construct a `Score` for the current player winning in `turn_count_win`
  /// moves.
  pub const fn win(turn_count_win: u32) -> Self {
    debug_assert!(turn_count_win != 0);
    Score::new(true, 0, turn_count_win)
  }

  /// Construct a `Score` for the current player winning in `turn_count_win`
  /// moves, assuming there is no faster way to force a win.
  pub const fn optimal_win(turn_count_win: u32) -> Self {
    debug_assert!(turn_count_win != 0);
    Score::new(true, turn_count_win - 1, turn_count_win)
  }

  /// Construct a `Score` for the current player losing in `turn_count_lose`
  /// moves.
  pub const fn lose(turn_count_lose: u32) -> Self {
    debug_assert!(turn_count_lose != 0);
    Score::new(false, 0, turn_count_lose)
  }

  /// Construct a `Score` for the current player losing in `turn_count_lose`
  /// moves, assuming there is no faster way for the opponent to force a win.
  pub const fn optimal_lose(turn_count_lose: u32) -> Self {
    debug_assert!(turn_count_lose != 0);
    Score::new(false, turn_count_lose - 1, turn_count_lose)
  }

  /// Construct a `Score` for no possible forcing win in `turn_count_tie` moves.
  pub const fn tie(turn_count_tie: u32) -> Self {
    Score::new(false, turn_count_tie, 0)
  }

  /// Construct a `Score` for no possible forcing win in any number of moves
  /// into the future.
  pub const fn guaranteed_tie() -> Self {
    Score::tie(Self::MAX_TIE_DEPTH)
  }

  /// Used to mark a game state as an ancestor of the current tree being
  /// explored. Will be overwritten with the actual score once its calculation
  /// is finished.
  const fn ancestor() -> Self {
    Self::ANCESTOR
  }

  /// The maximum depth that this score is determined to.
  pub fn determined_depth(&self) -> u32 {
    self.turn_count_tie().max(self.turn_count_win())
  }

  /// The score of the game given `depth` moves to play.
  pub fn score_at_depth(&self, depth: u32) -> ScoreValue {
    if depth <= self.turn_count_tie() {
      ScoreValue::Tie
    } else if depth >= self.turn_count_win() {
      if self.cur_player_wins() {
        ScoreValue::CurrentPlayerWins
      } else {
        ScoreValue::OtherPlayerWins
      }
    } else {
      debug_assert!(false, "Attempted to resolve score at undiscovered depth");
      unsafe { unreachable_unchecked() }
    }
  }

  pub const fn cur_player_wins(&self) -> bool {
    (self.data & Self::CUR_PLAYER_WINS_MASK) != 0
  }

  pub const fn turn_count_tie(&self) -> u32 {
    (self.data & Self::TIE_MASK) >> Self::TIE_SHIFT
  }

  pub const fn turn_count_win(&self) -> u32 {
    (self.data & Self::WIN_MASK) >> Self::WIN_SHIFT
  }

  /// Transforms a score at a given state of the game to how that score would
  /// appear from the perspective of a game state one step before it.
  ///
  /// For example, if a winning move for one player has been found in n steps,
  /// then it is turned into a winning move for the other player in n + 1
  /// steps.
  pub fn backstep(&self) -> Self {
    debug_assert!(self.turn_count_win() < Self::MAX_WIN_DEPTH);
    let (_, tie_bits, win_bits) = self.unpack_unshifted();
    let winning = win_bits != 0;
    let guaranteed_tie = tie_bits == Self::TIE_MASK;

    let to_add = (winning as u32 * ((1 << Self::WIN_SHIFT) | Self::CUR_PLAYER_WINS_MASK))
      + (!guaranteed_tie as u32 * (1 << Self::TIE_SHIFT));
    Score { data: self.data.wrapping_add(to_add) }
  }

  /// Transforms a score at a given state of the game to how that score would
  /// appear from the perspective of a game state one step after it.
  ///
  /// For example, if a winning move for one player has been found in n steps,
  /// then it is turned into a winning move for the other player in n - 1
  /// steps.
  pub fn forwardstep(&self) -> Self {
    let (_, tie_bits, win_bits) = self.unpack_unshifted();
    let swap_player_turn = win_bits != 0;
    let deduct_winning_turns = win_bits > (1 << Self::WIN_SHIFT);
    let deduct_tied_turns = tie_bits != Self::TIE_MASK && tie_bits != 0;

    Self {
      data: self.data.wrapping_sub(
        (swap_player_turn as u32 * Self::CUR_PLAYER_WINS_MASK)
          + (deduct_winning_turns as u32 * (1 << Self::WIN_SHIFT))
          + (deduct_tied_turns as u32 * (1 << Self::TIE_SHIFT)),
      ),
    }
  }

  /// Merges the information contained in another score into this one. This
  /// assumes that the scores are compatible, i.e. they don't contain
  /// conflicting information.
  pub const fn merge(&self, other: Self) -> Self {
    debug_assert!(self.compatible(other));

    let (cur_player_wins1, tie1, win1) = self.unpack_unshifted();
    let (cur_player_wins2, tie2, win2) = other.unpack_unshifted();

    let tie = max_u32(tie1, tie2);

    let win_one: u32 = 1 << Self::WIN_SHIFT;
    let win = min_u32(win1.wrapping_sub(win_one), win2.wrapping_sub(win_one)).wrapping_add(win_one);

    let cur_player_wins = cur_player_wins1 | cur_player_wins2;

    Score { data: tie + win + cur_player_wins }
  }

  /// True if this score can be used in place of a search that goes
  /// `search_depth` moves deep (i.e. this score will equal the score calculated
  /// by a full search this deep).
  pub const fn determined(&self, search_depth: u32) -> bool {
    let (_, turn_count_tie, turn_count_win) = Self::unpack(self.data);
    (turn_count_win != 0 && search_depth >= turn_count_win) || search_depth <= turn_count_tie
  }

  /// Returns true if the two scores don't contain conflicting information, i.e.
  /// they are compatible. If true, the scores can be safely `Score::merge`d.
  pub const fn compatible(&self, other: Score) -> bool {
    let (cur_player_wins1, turn_count_tie1, turn_count_win1) = Self::unpack(self.data);
    let (cur_player_wins2, turn_count_tie2, turn_count_win2) = Self::unpack(other.data);

    let tc_win1 = if turn_count_win1 == 0 {
      u32::MAX
    } else {
      turn_count_win1
    };
    let tc_win2 = if turn_count_win2 == 0 {
      u32::MAX
    } else {
      turn_count_win2
    };
    let score1 = if turn_count_win1 == 0 {
      cur_player_wins2
    } else {
      cur_player_wins1
    };
    let score2 = if turn_count_win2 == 0 {
      cur_player_wins1
    } else {
      cur_player_wins2
    };

    tc_win1 > turn_count_tie2 && tc_win2 > turn_count_tie1 && score1 == score2
  }

  /// True if this score is better than `other` for the current player.
  pub const fn better(&self, other: Score) -> bool {
    let (cur_player_wins1, turn_count_tie1, turn_count_win1) = Self::unpack(self.data);
    let (cur_player_wins2, turn_count_tie2, turn_count_win2) = Self::unpack(other.data);

    if turn_count_win2 != 0 {
      if cur_player_wins2 {
        // If both scores were wins, the better is the one with the shortest
        // path to victory.
        turn_count_win1 != 0 && cur_player_wins1 && turn_count_win1 < turn_count_win2
      } else {
        // If both scores are losses, the better is the one with the longest
        // path to losing.
        turn_count_win1 == 0 || cur_player_wins1 || turn_count_win1 > turn_count_win2
      }
    } else if turn_count_win1 != 0 {
      // If `other` is a tie and `this` is not, this is only better if it's a
      // win.
      cur_player_wins1
    } else {
      // If both scores were ties, the better is the score with the shortest
      // discovered tie depth.
      turn_count_tie1 < turn_count_tie2
    }
  }

  /// Constructs a score for a game state where not all possible next moves were
  /// explored. This sets `turn_count_tie` to 1, since we can't prove that there
  /// is no forced win out to any depth other than 1, since depth 1 is
  /// preemptively checked for immediate wins.
  pub fn break_early(&self) -> Self {
    debug_assert_ne!(self.turn_count_win(), 0);
    Score::new(self.cur_player_wins(), 0, self.turn_count_win())
  }

  const fn pack(cur_player_wins: bool, turn_count_tie: u32, turn_count_win: u32) -> u32 {
    debug_assert!(turn_count_tie <= Self::MAX_TIE_DEPTH);
    debug_assert!(turn_count_win <= Self::MAX_WIN_DEPTH);

    ((cur_player_wins as u32) << Self::CUR_PLAYER_WINS_SHIFT)
      + (turn_count_tie << Self::TIE_SHIFT)
      + (turn_count_win << Self::WIN_SHIFT)
  }

  const fn unpack_unshifted(&self) -> (u32, u32, u32) {
    (
      self.data & Self::CUR_PLAYER_WINS_MASK,
      self.data & Self::TIE_MASK,
      self.data & Self::WIN_MASK,
    )
  }

  const fn unpack(data: u32) -> (bool, u32, u32) {
    (
      (data & Self::CUR_PLAYER_WINS_MASK) != 0,
      (data & Self::TIE_MASK) >> Self::TIE_SHIFT,
      (data & Self::WIN_MASK) >> Self::WIN_SHIFT,
    )
  }
}

impl PartialEq for Score {
  fn eq(&self, other: &Self) -> bool {
    self.data == other.data
  }
}

impl Eq for Score {}

impl PartialOrd for Score {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Score {
  fn cmp(&self, other: &Self) -> Ordering {
    if self.better(*other) {
      debug_assert!(!other.better(*self));
      Ordering::Greater
    } else if self == other {
      debug_assert!(!other.better(*self));
      Ordering::Equal
    } else {
      debug_assert!(other.better(*self));
      Ordering::Less
    }
  }
}

impl Debug for Score {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self)
  }
}

impl Display for Score {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let (cur_player_wins, turn_count_tie, turn_count_win) = Self::unpack(self.data);

    if self == &Self::ancestor() {
      write!(f, "[ancestor]")
    } else if turn_count_win == 0 {
      if turn_count_tie == Self::MAX_TIE_DEPTH {
        write!(f, "[tie:∞]")
      } else {
        write!(f, "[tie:{turn_count_tie}]")
      }
    } else {
      write!(
        f,
        "[tie:{turn_count_tie},{}:{turn_count_win}]",
        if cur_player_wins { "cur" } else { "oth" }
      )
    }
  }
}

#[cfg(test)]
mod tests {
  use crate::Score;

  use googletest::{gtest, prelude::*};

  fn opposite_score(score: Score) -> Score {
    if score.is_tie() || score.is_ancestor() {
      return score;
    }

    Score::new(
      !score.cur_player_wins(),
      score.turn_count_tie(),
      score.turn_count_win(),
    )
  }

  fn check_compatible(s1: Score, s2: Score) {
    assert!(s1.compatible(s2), "{s1} vs {s2}");
    assert!(s2.compatible(s1), "{s2} vs {s1}");

    let opposite_s1 = opposite_score(s1);
    let opposite_s2 = opposite_score(s2);
    assert!(
      opposite_s1.compatible(opposite_s2),
      "{opposite_s1} vs {opposite_s2}"
    );
    assert!(
      opposite_s2.compatible(opposite_s1),
      "{opposite_s2} vs {opposite_s1}"
    );
  }

  fn check_incompatible(s1: Score, s2: Score) {
    assert!(!s1.compatible(s2), "{s1} vs {s2}");
    assert!(!s2.compatible(s1), "{s2} vs {s1}");

    let opposite_s1 = opposite_score(s1);
    let opposite_s2 = opposite_score(s2);
    assert!(
      !opposite_s1.compatible(opposite_s2),
      "{opposite_s1} vs {opposite_s2}"
    );
    assert!(
      !opposite_s2.compatible(opposite_s1),
      "{opposite_s2} vs {opposite_s1}"
    );
  }

  fn check_merge_eq(s1: Score, s2: Score, expected: Score) {
    assert_eq!(s1.merge(s2), expected, "Merging {s1} and {s2}");
    assert_eq!(s2.merge(s1), expected, "Merging {s2} and {s1}");

    let opposite_s1 = opposite_score(s1);
    let opposite_s2 = opposite_score(s2);
    let opposite_expected = opposite_score(expected);
    assert_eq!(
      opposite_s1.merge(opposite_s2),
      opposite_expected,
      "Merging {opposite_s1} and {opposite_s2}"
    );
    assert_eq!(
      opposite_s2.merge(opposite_s1),
      opposite_expected,
      "Merging {opposite_s2} and {opposite_s1}"
    );
  }

  #[test]
  fn test_compatible() {
    // Guaranteed tie is incompatible with anything that isn't a tie.
    check_compatible(Score::guaranteed_tie(), Score::guaranteed_tie());
    check_compatible(Score::guaranteed_tie(), Score::tie(10));
    check_compatible(Score::guaranteed_tie(), Score::no_info());

    check_incompatible(Score::guaranteed_tie(), Score::win(1));
    check_incompatible(Score::guaranteed_tie(), Score::lose(1));
    check_incompatible(Score::guaranteed_tie(), Score::win(10));
    check_incompatible(Score::guaranteed_tie(), Score::lose(10));

    // Scores are compatible if they have the same winner and don't disagree on
    // tie/win regions.
    check_compatible(Score::new(true, 10, 20), Score::new(true, 5, 40));
    check_compatible(Score::new(true, 5, 20), Score::new(true, 10, 40));
    check_compatible(Score::new(true, 10, 20), Score::new(true, 10, 40));
    check_compatible(Score::new(true, 5, 20), Score::new(true, 10, 20));
    check_compatible(Score::win(10), Score::win(20));
    check_compatible(Score::win(10), Score::new(true, 5, 20));

    // Scores with overlapping tied/win regions are incompatible.
    check_incompatible(Score::new(true, 0, 20), Score::new(true, 30, 40));
    check_incompatible(Score::new(true, 0, 20), Score::new(true, 20, 40));

    // Scores with different winners are always incompatible.
    check_incompatible(Score::new(true, 0, 20), Score::new(false, 30, 40));
    check_incompatible(Score::new(true, 0, 20), Score::new(false, 20, 40));
    check_incompatible(Score::new(true, 0, 20), Score::new(false, 0, 40));
    check_incompatible(Score::new(true, 0, 20), Score::new(false, 0, 20));
  }

  #[test]
  fn test_merge() {
    // Merging no_info with anything doesn't change the score.
    check_merge_eq(Score::no_info(), Score::win(10), Score::win(10));
    check_merge_eq(Score::no_info(), Score::lose(10), Score::lose(10));
    check_merge_eq(Score::no_info(), Score::tie(10), Score::tie(10));
    check_merge_eq(
      Score::no_info(),
      Score::optimal_win(10),
      Score::optimal_win(10),
    );
    check_merge_eq(
      Score::no_info(),
      Score::optimal_lose(10),
      Score::optimal_lose(10),
    );

    // Merging two wins/loses results in the smaller of the two.
    check_merge_eq(Score::win(10), Score::win(5), Score::win(5));
    check_merge_eq(
      Score::new(true, 5, 40),
      Score::new(true, 10, 20),
      Score::new(true, 10, 20),
    );
    check_merge_eq(
      Score::new(true, 5, 20),
      Score::new(true, 10, 40),
      Score::new(true, 10, 20),
    );

    // Merging a tie and a win results in a win
    check_merge_eq(Score::win(10), Score::tie(5), Score::new(true, 5, 10));
    check_merge_eq(
      Score::new(true, 5, 20),
      Score::tie(10),
      Score::new(true, 10, 20),
    );
  }

  #[gtest]
  fn test_backstep() {
    expect_eq!(Score::win(1).backstep(), Score::optimal_lose(2));
    expect_eq!(Score::lose(1).backstep(), Score::optimal_win(2));

    expect_eq!(Score::no_info().backstep(), Score::tie(1));
    expect_eq!(Score::guaranteed_tie().backstep(), Score::guaranteed_tie());
  }

  #[gtest]
  fn test_forwardstep() {
    expect_eq!(Score::win(2).forwardstep(), Score::lose(1));
    expect_eq!(Score::lose(2).forwardstep(), Score::win(1));

    expect_eq!(Score::win(1).forwardstep(), Score::lose(1));
    expect_eq!(Score::lose(1).forwardstep(), Score::win(1));

    expect_eq!(Score::no_info().forwardstep(), Score::no_info());
    expect_eq!(Score::tie(1).forwardstep(), Score::no_info());
    expect_eq!(
      Score::guaranteed_tie().forwardstep(),
      Score::guaranteed_tie()
    );
  }
}
