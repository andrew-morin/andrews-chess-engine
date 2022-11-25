#![feature(const_mut_refs)]
#![feature(test)]

#[macro_use]
extern crate serde_big_array;
extern crate wasm_bindgen;

mod board;
mod engine;

use board::GameState;
use board::types::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn get_game_state_from_fen(fen: &str) -> JsValue {
  let initial_game_state = board::fen_util::get_game_state_from_fen(fen);
  JsValue::from_serde(&initial_game_state).unwrap()
}

#[wasm_bindgen]
pub fn get_initial_game_state() -> JsValue {
  let initial_game_state = GameState::default();
  let result = JsValue::from_serde(&initial_game_state).unwrap();
  result
}

#[wasm_bindgen]
pub fn convert_game_state_to_squares(game_state: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let squares: Vec<(Color, Piece)> = (0..64).map(|index| game_state.board.get_square(index)).collect();
  JsValue::from_serde(&squares).unwrap()
}

#[wasm_bindgen]
pub fn get_square_at_index(game_state: JsValue, index: usize) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let square = game_state.board.get_square(index);
  JsValue::from_serde(&square).unwrap()
}

#[wasm_bindgen]
pub fn get_pseudo_legal_moves(game_state: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let moves = game_state.generate_pseudo_legal_moves();
  JsValue::from_serde(&moves).unwrap()
}

#[wasm_bindgen]
pub fn get_next_legal_game_states(game_state: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let moves = game_state.generate_legal_moves();
  JsValue::from_serde(&moves).unwrap()
}

#[wasm_bindgen]
pub fn perform_move(game_state: JsValue, next_move: JsValue) -> JsValue {
  let mut game_state: GameState = game_state.into_serde().unwrap();
  let next_move: Move = next_move.into_serde().unwrap();
  game_state.perform_move(next_move);
  JsValue::from_serde(&game_state).unwrap()
}

#[wasm_bindgen]
pub struct InCheckReturn (pub bool, pub usize);

#[wasm_bindgen]
pub fn in_check(game_state: JsValue) -> InCheckReturn {
  let game_state: GameState = game_state.into_serde().unwrap();
  let (b, i) = game_state.is_in_check();
  InCheckReturn(b, i)
}
