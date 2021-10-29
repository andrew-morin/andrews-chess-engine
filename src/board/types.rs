#[repr(u8)]
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Color {
  Empty,
  White,
  Black,
}

#[repr(u8)]
#[derive(Debug)]
pub enum Piece {
  Empty,
  Pawn,
  Bishop,
  Knight,
  Rook,
  Queen,
  King,
}

#[derive(Debug)]
pub struct Square {
  pub empty: bool,
  pub color: Color,
  pub piece: Piece,
}

pub type Board = [Square; 64];

pub struct Move {
  pub from: usize,
  pub to: usize,
  pub capture: bool,
  pub castle: bool,
}

impl Move {
  pub fn new(from: usize, to: usize) -> Move {
    Move { capture: false, castle: false, from, to }
  }
  pub fn capture(from: usize, to: usize) -> Move {
    Move { capture: true, castle: false, from, to }
  }
  pub fn castle(from: usize, to: usize) -> Move {
    Move { capture: false, castle: true, from, to }
  }
}

pub struct GameState {
  pub board: Board,
  pub turn: Color,
}
