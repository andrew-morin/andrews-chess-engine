#[macro_use]
extern crate serde_big_array;
extern crate wasm_bindgen;

mod board;
mod engine;

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
  JsValue::from_serde(&initial_game_state).unwrap()
}

#[wasm_bindgen]
pub fn get_pseudo_legal_moves(game_state: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let moves = board::generate_pseudo_legal_moves(&game_state);
  return JsValue::from_serde(&moves).unwrap();
}

#[wasm_bindgen]
pub fn get_legal_moves(game_state: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let moves = board::generate_legal_moves(&game_state);
  return JsValue::from_serde(&moves).unwrap();
}

#[wasm_bindgen]
pub fn perform_move(game_state: JsValue, next_move: JsValue) -> JsValue {
  let game_state: GameState = game_state.into_serde().unwrap();
  let next_move: Move = next_move.into_serde().unwrap();
  let game_state = board::perform_move(game_state, next_move);
  return JsValue::from_serde(&game_state).unwrap();
}

#[wasm_bindgen]
pub struct InCheckReturn (pub bool, pub usize);

#[wasm_bindgen]
pub fn in_check(game_state: JsValue) -> InCheckReturn {
  let game_state: GameState = game_state.into_serde().unwrap();
  let (b, i) = board::in_check(&game_state, game_state.turn);
  InCheckReturn(b, i)
}
