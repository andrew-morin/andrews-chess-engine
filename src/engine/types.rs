use crate::board::types::{Board, Color};

pub struct Move {
  from: (u8, u8),
  to: (u8, u8),
  capture: bool,
  castle: bool,
}

pub struct GameState {
  board: Board,
  turn: Color,
}