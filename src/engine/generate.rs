use std::cmp::Ordering;

use crate::board::{
    types::{Board, Color, Move, Piece},
    GameState,
};
use rand::prelude::SliceRandom;

// Values based on AlphaZero: https://arxiv.org/pdf/2009.04374.pdf (page 16)
const PAWN_VALUE: i32 = 100;
const KNIGHT_VALUE: i32 = 305;
const BISHOP_VALUE: i32 = 333;
const ROOK_VALUE: i32 = 563;
const QUEEN_VALUE: i32 = 650;
const EVALED_PIECES: [Piece; 5] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
];
const MAX_PLY_DEPTH: u32 = 4;

impl Piece {
    fn get_value(self: &Piece) -> i32 {
        match self {
            Piece::Pawn => PAWN_VALUE,
            Piece::Knight => KNIGHT_VALUE,
            Piece::Bishop => BISHOP_VALUE,
            Piece::Rook => ROOK_VALUE,
            Piece::Queen => QUEEN_VALUE,
            _ => 0,
        }
    }
}

pub fn search(game_state: &GameState) -> Option<Move> {
    search_at_depth(game_state, MAX_PLY_DEPTH)
}

pub fn search_at_depth(game_state: &GameState, depth: u32) -> Option<Move> {
    inner_search(game_state, depth).map(|(state, _eval)| {
        evaluate(&state);
        state.move_list[game_state.move_list.len()]
    })
}

fn inner_search(game_state: &GameState, depth: u32) -> Option<(GameState, i32)> {
    if depth == 0 {
        panic!("depth must be at least 1");
    }
    let states = game_state.generate_legal_states();
    if states.is_empty() {
        return None;
    }
    let mut best_states = vec![];
    let mut best_side_eval = i32::MIN;
    let sign: i32 = match game_state.turn {
        Color::Black => -1,
        _ => 1,
    };
    for mut state in states {
        let side_eval;
        if depth > 1 {
            let new_state = inner_search(&state, depth - 1);
            if let Some((new_state, new_eval)) = new_state {
                state = new_state;
                side_eval = sign * new_eval;
            } else {
                // If there is no best move, then it must be checkmate or stalemate
                side_eval = if game_state.is_in_check() {
                    -i32::MAX
                } else {
                    0
                };
            }
        } else {
            side_eval = sign * evaluate(&state);
        }
        match side_eval.cmp(&best_side_eval) {
            Ordering::Greater => {
                best_states = vec![state];
                best_side_eval = side_eval;
            }
            Ordering::Equal => {
                best_states.push(state);
            }
            Ordering::Less => {}
        }
    }
    let best_state = best_states.choose(&mut rand::thread_rng());
    best_state.map(|best_state| (best_state.clone(), sign * best_side_eval))
}

fn evaluate(game_state: &GameState) -> i32 {
    if game_state.generate_legal_states().is_empty() {
        return get_no_move_eval(game_state);
    }
    let board = &game_state.board;

    let mut white_eval = 0;
    for piece in EVALED_PIECES {
        white_eval += get_piece_eval(board, Color::White, piece);
    }

    let mut black_eval = 0;
    for piece in EVALED_PIECES {
        black_eval += get_piece_eval(board, Color::Black, piece);
    }

    white_eval - black_eval
}

// Assumes there are no legal moves. Make sure to check that first!
fn get_no_move_eval(game_state: &GameState) -> i32 {
    if game_state.is_in_check() {
        match game_state.turn {
            Color::White => -i32::MAX,
            Color::Black => i32::MAX,
            _ => unreachable!(),
        }
    } else {
        0
    }
}

fn get_piece_eval(board: &Board, color: Color, piece: Piece) -> i32 {
    let bits = board.get_color_bitmask(color) & board.get_piece_bitmask(piece);
    // We can never have more than 16 bits on, so this cast is safe
    piece.get_value() * bits.count_ones() as i32
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::board::fen_util::*;
    use crate::board::GameState;

    #[test]
    fn search_start_pos() {
        let m = search_at_depth(&GameState::default(), 2);
        assert!(m.is_some());
    }

    #[test]
    fn search_capture() {
        let state =
            get_game_state_from_fen("rnbqkbnr/pppp1ppp/8/4p3/3P4/8/PPP1PPPP/RNBQKBNR w KQkq - 0 1");
        let opt_m = search_at_depth(&state, 2);
        assert!(opt_m.is_some());
        let m = opt_m.unwrap();
        let expected = Move::capture(35, 28);
        assert_eq!(expected, m);
    }
}

#[cfg(test)]
mod benchmark_tests {
    extern crate test;

    use crate::board::GameState;

    use super::search_at_depth;
    use test::Bencher;

    #[bench]
    fn search_depth_1_start_position(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| search_at_depth(&game_state, 1));
    }

    // These tests are slow, so ignore them by default

    #[bench]
    #[ignore]
    fn search_depth_2_start_position(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| search_at_depth(&game_state, 2));
    }

    #[bench]
    #[ignore]
    fn search_depth_4_start_position(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| search_at_depth(&game_state, 4));
    }

    #[bench]
    #[ignore]
    fn search_depth_5_start_position(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| search_at_depth(&game_state, 5));
    }
}
