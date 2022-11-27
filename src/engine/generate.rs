use crate::board::{types::Move, GameState};

use rand::seq::SliceRandom;

pub fn search(game_state: &GameState) -> Option<Move> {
    let moves = game_state.generate_legal_moves();
    let state = moves.choose(&mut rand::thread_rng());
    if let Some(state) = state {
        return Option::Some(state.move_list[state.move_list.len() - 1]);
    }
    None
}
