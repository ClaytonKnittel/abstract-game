use crate::error::GameInterfaceResult;

pub trait InputReader {
  type Output;

  fn next_input(&mut self) -> GameInterfaceResult<Self::Output>;
}
