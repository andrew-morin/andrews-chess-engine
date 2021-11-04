use serde::{Serialize, Deserialize};
use super::constants::INITIAL_BOARD;

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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Square {
  pub empty: bool,
  pub color: Color,
  pub piece: Piece,
}

pub type Board = [Square; 64];

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Move {
  pub from: usize,
  pub to: usize,
  pub capture: bool,
  pub en_passant: bool,
  pub castle: bool,
  pub two_square_pawn_move: bool,
}

impl Move {
  pub fn new(from: usize, to: usize) -> Move {
    Move { from, to, ..Default::default() }
  }
  pub fn capture(from: usize, to: usize) -> Move {
    Move { from, to, capture: true, ..Default::default() }
  }
  pub fn en_passant(from: usize, to: usize) -> Move {
    Move { from, to, capture: true, en_passant: true, ..Default::default() }
  }
  pub fn castle(from: usize, to: usize) -> Move {
    Move { from, to, castle: true, ..Default::default() }
  }
  pub fn two_square_pawn_move(from: usize, to: usize) -> Move {
    Move { from, to, two_square_pawn_move: true, ..Default::default() }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CastleAvailability {
  pub white_kingside: bool,
  pub white_queenside: bool,
  pub black_kingside: bool,
  pub black_queenside: bool,
}

impl Default for CastleAvailability {
  fn default() -> Self {
    CastleAvailability {
      white_kingside: true,
      white_queenside: true,
      black_kingside: true,
      black_queenside: true,
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
  #[serde(with = "BigArray")]
  pub board: Board,
  pub turn: Color,
  pub move_list: Vec<Move>,
  pub castle: CastleAvailability,
  pub en_passant_index: Option<usize>,
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      board: INITIAL_BOARD,
      turn: Color::White,
      move_list: vec!(),
      castle: Default::default(),
      en_passant_index: None,
    }
  }
}
