extern crate wasm_bindgen;

mod board;
mod engine;

use board::types::{Board, Color, Piece, Square};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn print_board() -> String {
  board_to_string(board::constants::INITIAL_BOARD)
}

fn board_to_string(board: Board) -> String {
  let mut board_str = String::new();
  for (index, square) in board.iter().enumerate() {
    board_str.push(get_fen_char_from_square(square));

    // last square in the rank
    if index % 8 == 7 {
      board_str.push_str("\n\n");
    } else {
      board_str.push_str(" | ");
    }
  }
  board_str
}

fn get_fen_char_from_square(square: &Square) -> char {
  if square.empty == true {
    return ' ';
  }
  match square.color {
    Color::Black => match square.piece {
      Piece::Pawn   => 'p',
      Piece::Bishop => 'b',
      Piece::Knight => 'n',
      Piece::Rook   => 'r',
      Piece::Queen  => 'q',
      Piece::King   => 'k',
      Piece::Empty  => ' ',
    },
    Color::White => match square.piece {
      Piece::Pawn   => 'P',
      Piece::Bishop => 'B',
      Piece::Knight => 'N',
      Piece::Rook   => 'R',
      Piece::Queen  => 'Q',
      Piece::King   => 'K',
      Piece::Empty  => ' ',
    },
    Color::Empty => ' ',
  }
}
