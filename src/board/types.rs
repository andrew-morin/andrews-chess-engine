#[derive(Debug)]
pub enum Color {
  Empty,
  White,
  Black,
}

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
