#[macro_use]
extern crate serde_big_array;
extern crate wasm_bindgen;

mod board;
mod engine;

use board::constants::*;
use board::types::*;
use board::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn print_board() -> String {
  board_to_fen_string(board::constants::INITIAL_BOARD)
}

#[wasm_bindgen]
pub fn get_initial_game_state() -> JsValue {
  let initial_game_state = INITIAL_GAME_STATE;
  JsValue::from_serde(&initial_game_state).unwrap()
}

#[wasm_bindgen]
pub fn get_pseudo_legal_moves(game_state: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let moves = generate_pseudo_legal_moves(&game_state);
  return JsValue::from_serde(&moves).unwrap();
}

fn board_to_fen_string(board: Board) -> String {
  let mut board_str = String::new();
  let mut space_count = 0;
  for (index, square) in board.iter().enumerate() {
    let letter = get_fen_char_from_square(square);
    if letter == ' ' {
      space_count += 1;
    } else {
      if space_count > 0 {
        board_str.push_str(&space_count.to_string());
        space_count = 0;
      }
      board_str.push(get_fen_char_from_square(square));
    }

    // last square in the rank, but not last rank
    if index % 8 == 7 && index < 63 {
      if space_count > 0 {
        board_str.push_str(&space_count.to_string());
        space_count = 0;
      }
      board_str.push_str("/");
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
