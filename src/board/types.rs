use std::fmt;

#[repr(u8)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Color {
  Empty,
  White,
  Black,
}

impl Default for Color {
  fn default() -> Self { Color::Empty }
}

#[repr(u8)]
#[derive(Debug)]
#[derive(PartialEq)]
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

#[derive(Debug)]
#[derive(Default)]
pub struct Square {
  pub empty: bool,
  pub color: Color,
  pub piece: Piece,
}

pub type Board = [Square; 64];

#[derive(Debug)]
pub struct Move {
  pub from: usize,
  pub to: usize,
  pub capture: bool,
  pub castle: bool,
  pub two_square_pawn_move: bool,
}

impl Move {
  pub fn new(from: usize, to: usize) -> Move {
    Move { from, to, capture: false, castle: false, two_square_pawn_move: false }
  }
  pub fn capture(from: usize, to: usize) -> Move {
    Move { from, to, capture: true, castle: false, two_square_pawn_move: false }
  }
  pub fn castle(from: usize, to: usize) -> Move {
    Move { from, to, capture: false, castle: true, two_square_pawn_move: false }
  }
  pub fn two_square_pawn_move(from: usize, to: usize) -> Move {
    Move { from, to, capture: false, castle: false, two_square_pawn_move: true }
  }
}

pub struct GameState {
  pub board: Board,
  pub turn: Color,
}
