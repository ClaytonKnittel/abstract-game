use std::{
  cmp::Ordering,
  fmt::{Debug, Display},
  hint::unreachable_unchecked,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreValue {
  CurrentPlayerWins,
  OtherPlayerWins,
  Tie,
}

#[derive(Clone, Copy)]
pub struct Score {
  /// Layout:
  /// ```text
  ///          31         30 -  23  22     -    12   11     -     0
  /// +------------------+--------+----------------+----------------+
  /// | cur player wins? | unused | turn count win | turn count tie |
  /// +------------------+--------+----------------+----------------+
  /// ```
  pub(crate) data: u32,
}

impl Score {
  const TIE_BITS: u32 = 11;
  const TIE_SHIFT: u32 = 0;
  const MAX_TIE_DEPTH: u32 = (1 << Self::TIE_BITS) - 1;
  const TIE_MASK: u32 = Self::MAX_TIE_DEPTH << Self::TIE_SHIFT;

  const WIN_BITS: u32 = 11;
  const WIN_SHIFT: u32 = Self::TIE_SHIFT + Self::TIE_BITS;
  const MAX_WIN_DEPTH: u32 = (1 << Self::WIN_BITS) - 1;
  const WIN_MASK: u32 = Self::MAX_WIN_DEPTH << Self::WIN_SHIFT;

  const UNUSED_BITS: u32 = 9;
  const UNUSED_SHIFT: u32 = Self::WIN_SHIFT + Self::WIN_BITS;

  const CUR_PLAYER_WINS_SHIFT: u32 = Self::UNUSED_SHIFT + Self::UNUSED_BITS;
  const CUR_PLAYER_WINS_MASK: u32 = 1 << Self::CUR_PLAYER_WINS_SHIFT;

  /// A `Score` that contains no information.
  const NO_INFO: Score = Score::new(false, 0, 0);

  /// Mark the current player as winning with turn_count_win_ = 0, which is an
  /// impossible state to be in.
  const ANCESTOR: Score = Score { data: Self::CUR_PLAYER_WINS_MASK };

  const fn new(cur_player_wins: bool, turn_count_tie: u32, turn_count_win: u32) -> Self {
    debug_assert!(turn_count_tie <= Self::MAX_TIE_DEPTH);
    debug_assert!(turn_count_win < Self::MAX_WIN_DEPTH);
    debug_assert!(
      !cur_player_wins || turn_count_win != 0,
      "If turn_count_win == 0, then this is a tie, and cur_player_wins must be false."
    );
    Self {
      data: Self::pack(
        cur_player_wins,
        turn_count_tie,
        if turn_count_win == 0 {
          Self::MAX_WIN_DEPTH
        } else {
          turn_count_win - 1
        },
      ),
    }
  }

  /// Returns true if this score contains no info.
  pub const fn has_no_info(&self) -> bool {
    self.data == Self::NO_INFO.data
  }

  pub const fn is_tie(&self) -> bool {
    (self.data & Self::WIN_MASK) == Self::WIN_MASK
  }

  pub const fn is_guaranteed_tie(&self) -> bool {
    (self.data & Self::TIE_MASK) == Self::TIE_MASK
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
    let (_, tie, win) = Self::unpack(self.data + (1 << Self::WIN_SHIFT));
    tie.max(win)
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

  const fn cur_player_wins(&self) -> bool {
    (self.data & Self::CUR_PLAYER_WINS_MASK) != 0
  }

  const fn turn_count_tie(&self) -> u32 {
    (self.data & Self::TIE_MASK) >> Self::TIE_SHIFT
  }

  const fn turn_count_win(&self) -> u32 {
    debug_assert!(!self.is_tie());
    ((self.data + (1 << Self::WIN_SHIFT)) & Self::WIN_MASK) >> Self::WIN_SHIFT
  }

  /// Transforms a score at a given state of the game to how that score would
  /// appear from the perspective of a game state one step before it.
  ///
  /// For example, if a winning move for one player has been found in n steps,
  /// then it is turned into a winning move for the other player in n + 1
  /// steps.
  pub fn backstep(&self) -> Self {
    debug_assert!(self.is_tie() || self.turn_count_win() < Self::MAX_WIN_DEPTH);
    let to_add = (!self.is_tie() as u32 * ((1 << Self::WIN_SHIFT) | Self::CUR_PLAYER_WINS_MASK))
      + (!self.is_guaranteed_tie() as u32 * (1 << Self::TIE_SHIFT));
    Score { data: self.data.wrapping_add(to_add) }
  }

  /// Transforms a score at a given state of the game to how that score would
  /// appear from the perspective of a game state one step after it.
  ///
  /// For example, if a winning move for one player has been found in n steps,
  /// then it is turned into a winning move for the other player in n - 1
  /// steps.
  pub fn forwardstep(&self) -> Self {
    let (_, tie_bits, win_bits) = Self::unpack_unshifted(self.data);
    let swap_player_turn = !self.is_tie();
    let deduct_winning_turns = swap_player_turn && win_bits != 0;
    let deduct_tied_turns = !self.is_guaranteed_tie() && tie_bits != 0;

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
  pub fn merge(&self, other: Self) -> Self {
    debug_assert!(self.compatible(other));

    let (cur_player_wins1, tie1, win1) = Self::unpack_unshifted(self.data);
    let (cur_player_wins2, tie2, win2) = Self::unpack_unshifted(other.data);

    let tie = tie1.max(tie2);
    let win = win1.min(win2);
    let cur_player_wins = cur_player_wins1 | cur_player_wins2;

    Score { data: tie + win + cur_player_wins }
  }

  /// True if this score can be used in place of a search that goes
  /// `search_depth` moves deep (i.e. this score will equal the score calculated
  /// by a full search this deep).
  pub fn determined(&self, search_depth: u32) -> bool {
    let (_, turn_count_tie, turn_count_win) = Self::unpack(self.data);
    search_depth > turn_count_win || search_depth <= turn_count_tie
  }

  /// Returns true if the two scores don't contain conflicting information, i.e.
  /// they are compatible. If true, the scores can be safely `Score::merge`d.
  pub fn compatible(&self, other: Score) -> bool {
    let tie_to_win_shift = Self::WIN_SHIFT - Self::TIE_SHIFT;

    let (cur_player_wins1, tie1, win1) = Self::unpack_unshifted(self.data);
    let (cur_player_wins2, tie2, win2) = Self::unpack_unshifted(other.data);

    let agree = self.is_tie() || other.is_tie() || cur_player_wins1 == cur_player_wins2;

    win1 >= (tie2 << tie_to_win_shift) && win2 >= (tie1 << tie_to_win_shift) && agree
  }

  /// True if this score is better than `other` for the current player.
  pub fn better(&self, other: Score) -> bool {
    let transform_data = |data: u32| -> u32 {
      let cpw_to_win_shift = Self::CUR_PLAYER_WINS_SHIFT - Self::WIN_SHIFT;
      let shifted_bits = (data as i32 >> cpw_to_win_shift) as u32;
      let mask = shifted_bits & Self::WIN_MASK;
      data ^ mask
    };

    let data1 = transform_data(self.data);
    let data2 = transform_data(other.data);

    data1 > data2
  }

  /// Constructs a score for a game state where not all possible next moves were
  /// explored. This sets `turn_count_tie` to 0, since we can't prove that there
  /// is no forced win out to any depth.
  pub fn break_early(&self) -> Self {
    Score { data: self.data & !Self::TIE_MASK }
  }

  const fn pack(cur_player_wins: bool, turn_count_tie: u32, turn_count_win: u32) -> u32 {
    debug_assert!(turn_count_tie <= Self::MAX_TIE_DEPTH);
    debug_assert!(turn_count_win <= Self::MAX_WIN_DEPTH);

    ((cur_player_wins as u32) << Self::CUR_PLAYER_WINS_SHIFT)
      + (turn_count_tie << Self::TIE_SHIFT)
      + (turn_count_win << Self::WIN_SHIFT)
  }

  const fn unpack_unshifted(data: u32) -> (u32, u32, u32) {
    (
      data & Self::CUR_PLAYER_WINS_MASK,
      data & Self::TIE_MASK,
      data & Self::WIN_MASK,
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
  use crate::{Score, ScoreValue};

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

  #[gtest]
  fn test_determined() {
    expect_true!(Score::win(10).determined(10));
    expect_false!(Score::win(10).determined(9));
    expect_false!(Score::win(10).determined(1));
    expect_true!(Score::win(10).determined(0));
    expect_true!(Score::win(10).determined(100));

    expect_true!(Score::lose(10).determined(10));
    expect_false!(Score::lose(10).determined(9));
    expect_false!(Score::lose(10).determined(1));
    expect_true!(Score::lose(10).determined(0));
    expect_true!(Score::lose(10).determined(100));

    expect_true!(Score::optimal_win(10).determined(10));
    expect_true!(Score::optimal_win(10).determined(9));
    expect_true!(Score::optimal_win(10).determined(0));
    expect_true!(Score::optimal_win(10).determined(100));

    expect_true!(Score::optimal_lose(10).determined(10));
    expect_true!(Score::optimal_lose(10).determined(9));
    expect_true!(Score::optimal_lose(10).determined(0));
    expect_true!(Score::optimal_lose(10).determined(100));

    expect_true!(Score::guaranteed_tie().determined(0));
    expect_true!(Score::guaranteed_tie().determined(1));
    expect_true!(Score::guaranteed_tie().determined(10));

    expect_true!(Score::tie(5).determined(5));
    expect_true!(Score::tie(5).determined(1));
    expect_false!(Score::tie(5).determined(6));
    expect_false!(Score::tie(5).determined(100));
  }

  #[test]
  fn test_compatible() {
    // Guaranteed tie is incompatible with anything that isn't a tie.
    check_compatible(Score::guaranteed_tie(), Score::guaranteed_tie());
    check_compatible(Score::guaranteed_tie(), Score::tie(10));
    check_compatible(Score::guaranteed_tie(), Score::NO_INFO);

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
    check_merge_eq(Score::NO_INFO, Score::win(10), Score::win(10));
    check_merge_eq(Score::NO_INFO, Score::lose(10), Score::lose(10));
    check_merge_eq(Score::NO_INFO, Score::tie(10), Score::tie(10));
    check_merge_eq(
      Score::NO_INFO,
      Score::optimal_win(10),
      Score::optimal_win(10),
    );
    check_merge_eq(
      Score::NO_INFO,
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
  fn test_determined_depth() {
    expect_eq!(Score::win(3).determined_depth(), 3);
    expect_eq!(Score::optimal_win(3).determined_depth(), 3);
    expect_eq!(Score::lose(3).determined_depth(), 3);
    expect_eq!(Score::optimal_lose(3).determined_depth(), 3);
    expect_eq!(Score::tie(3).determined_depth(), 3);

    expect_eq!(Score::NO_INFO.determined_depth(), 0);
  }

  #[gtest]
  fn test_score_at_depth() {
    expect_eq!(
      Score::win(3).score_at_depth(3),
      ScoreValue::CurrentPlayerWins
    );
    expect_eq!(
      Score::win(3).score_at_depth(10),
      ScoreValue::CurrentPlayerWins
    );

    expect_eq!(
      Score::optimal_win(3).score_at_depth(3),
      ScoreValue::CurrentPlayerWins
    );
    expect_eq!(
      Score::optimal_win(3).score_at_depth(10),
      ScoreValue::CurrentPlayerWins
    );
    expect_eq!(Score::optimal_win(3).score_at_depth(2), ScoreValue::Tie);

    expect_eq!(
      Score::lose(3).score_at_depth(3),
      ScoreValue::OtherPlayerWins
    );
    expect_eq!(
      Score::lose(3).score_at_depth(10),
      ScoreValue::OtherPlayerWins
    );

    expect_eq!(
      Score::optimal_lose(3).score_at_depth(3),
      ScoreValue::OtherPlayerWins
    );
    expect_eq!(
      Score::optimal_lose(3).score_at_depth(10),
      ScoreValue::OtherPlayerWins
    );
    expect_eq!(Score::optimal_lose(3).score_at_depth(2), ScoreValue::Tie);

    expect_eq!(Score::tie(3).score_at_depth(3), ScoreValue::Tie);
    expect_eq!(Score::tie(3).score_at_depth(0), ScoreValue::Tie);

    expect_eq!(Score::guaranteed_tie().score_at_depth(100), ScoreValue::Tie);
    expect_eq!(Score::guaranteed_tie().score_at_depth(0), ScoreValue::Tie);

    expect_eq!(Score::NO_INFO.score_at_depth(0), ScoreValue::Tie);
  }

  #[gtest]
  fn test_backstep() {
    expect_eq!(Score::win(1).backstep(), Score::optimal_lose(2));
    expect_eq!(Score::lose(1).backstep(), Score::optimal_win(2));

    expect_eq!(Score::NO_INFO.backstep(), Score::tie(1));
    expect_eq!(Score::guaranteed_tie().backstep(), Score::guaranteed_tie());
  }

  #[gtest]
  fn test_forwardstep() {
    expect_eq!(Score::win(2).forwardstep(), Score::lose(1));
    expect_eq!(Score::lose(2).forwardstep(), Score::win(1));

    expect_eq!(Score::win(1).forwardstep(), Score::lose(1));
    expect_eq!(Score::lose(1).forwardstep(), Score::win(1));

    expect_eq!(Score::NO_INFO.forwardstep(), Score::NO_INFO);
    expect_eq!(Score::tie(1).forwardstep(), Score::NO_INFO);
    expect_eq!(
      Score::guaranteed_tie().forwardstep(),
      Score::guaranteed_tie()
    );
  }

  #[gtest]
  fn test_better() {
    // Winning is better than losing.
    expect_gt!(Score::win(1), Score::lose(1));
    expect_gt!(Score::win(100), Score::lose(1));

    // Winning is better than tying.
    expect_gt!(Score::win(1), Score::tie(1));
    expect_gt!(Score::win(1), Score::guaranteed_tie());

    // Winning is better than no info.
    expect_gt!(Score::win(1), Score::NO_INFO);

    // Winning in fewer moves is better than more moves.
    expect_gt!(Score::win(5), Score::win(10));

    // If the number of moves to a win is equal, prefer the move with a higher
    // discovered tie depth, which has a higher chance of pruning when
    // searching.
    expect_gt!(Score::optimal_win(5), Score::win(5));

    // Tying is better than losing.
    expect_gt!(Score::tie(1), Score::lose(1));
    expect_gt!(Score::tie(10), Score::lose(1));
    expect_gt!(Score::tie(1), Score::lose(10));

    // Given two ties, prefer the one with a deeper discovered depth.
    expect_gt!(Score::tie(10), Score::tie(5));
    expect_gt!(Score::guaranteed_tie(), Score::tie(10));
    expect_gt!(Score::tie(5), Score::NO_INFO);

    // Losing is worse than no info.
    expect_gt!(Score::NO_INFO, Score::lose(10));

    // Given two losing scores, prefer the deeper one.
    expect_gt!(Score::lose(10), Score::lose(5));

    // If both scores are losing to the same depth, prefer the one with a
    // higher discovered tie depth.
    expect_gt!(Score::optimal_lose(10), Score::lose(10));
  }

  #[gtest]
  fn test_break_early() {
    expect_eq!(Score::win(3).break_early(), Score::win(3));
    expect_eq!(Score::optimal_win(3).break_early(), Score::win(3));
    expect_eq!(Score::lose(3).break_early(), Score::lose(3));
    expect_eq!(Score::optimal_lose(3).break_early(), Score::lose(3));
    expect_eq!(Score::tie(5).break_early(), Score::NO_INFO);
    expect_eq!(Score::guaranteed_tie().break_early(), Score::NO_INFO);
    expect_eq!(Score::NO_INFO.break_early(), Score::NO_INFO);
  }
}
