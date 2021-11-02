use serde::{Serialize, Deserialize};

big_array! { BigArray; }

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Color {
  Empty,
  White,
  Black,
}

impl Color {
  pub fn opposite(&self) -> Color {
    match self {
      Color::White => Color::Black,
      Color::Black => Color::White,
      Color::Empty => Color::Empty,
    }
  }
}

impl Default for Color {
  fn default() -> Self { Color::Empty }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Piece {
  Empty,
  Pawn,
  Bishop,
  Knight,
  Rook,
  Queen,
  King,
}

impl Default for Piece {
  fn default() -> Self { Piece::Empty }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Square {
  pub empty: bool,
  pub color: Color,
  pub piece: Piece,
}

pub type Board = [Square; 64];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default)]
pub struct Move {
  pub from: usize,
  pub to: usize,
  pub capture: bool,
  pub castle: bool,
  pub captured_square: Square,
  pub two_square_pawn_move: bool,
}

impl Move {
  pub fn new(from: usize, to: usize) -> Move {
    Move { from, to, ..Default::default() }
  }
  pub fn capture(from: usize, to: usize, square: Square) -> Move {
    Move { from, to, capture: true, captured_square: square, ..Default::default() }
  }
  pub fn castle(from: usize, to: usize) -> Move {
    Move { from, to, castle: true, ..Default::default() }
  }
  pub fn two_square_pawn_move(from: usize, to: usize) -> Move {
    Move { from, to, two_square_pawn_move: true, ..Default::default() }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameState {
  #[serde(with = "BigArray")]
  pub board: Board,
  pub turn: Color,
  pub move_list: Vec<Move>,
}
