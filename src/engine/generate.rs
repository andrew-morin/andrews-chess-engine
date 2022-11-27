use std::cmp::Ordering;

use crate::{
    board::{
        types::{Board, Color, Move, Piece},
        GameState,
    },
    log,
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
    let states = game_state.generate_legal_moves();
    if states.is_empty() {
        return None;
    }
    let mut best_states = vec![&states[0]];
    let mut best_eval = evaluate(best_states[0]);
    let sign: i32 = match game_state.turn {
        Color::Black => -1,
        _ => 1,
    };
    for state in &states[1..] {
        let eval = sign * evaluate(state);
        // log!("{}", eval);
        match eval.cmp(&best_eval) {
            Ordering::Greater => {
                best_states = vec![state];
                best_eval = eval;
                log!("{}", eval);
            }
            Ordering::Equal => {
                best_states.push(state);
            }
            Ordering::Less => {}
        }
    }
    let best_state = best_states.choose(&mut rand::thread_rng());
    best_state.map(|state| state.move_list[state.move_list.len() - 1])
}

fn evaluate(game_state: &GameState) -> i32 {
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

fn get_piece_eval(board: &Board, color: Color, piece: Piece) -> i32 {
    let bits = board.get_color_bitmask(color) & board.get_piece_bitmask(piece);
    // We can never have all 32 bits on, so this cast is safe
    piece.get_value() * bits.count_ones() as i32
}
