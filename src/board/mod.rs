pub mod types;
pub mod constants;
pub mod fen_util;

use constants::*;
use types::{Color, GameState, Move, Piece, Square};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

const CARDINAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [1, 10];
const DIAGONAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [9, 11];
const ALL_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [1, 9, 10, 11];
const KNIGHT_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [8, 12, 19, 21];

pub fn in_check(game_state: &GameState, color: Color) -> (bool, usize) {
  let king_index = find_king(game_state, color);
  if let Some(king_index) = king_index {
    let attack_moves = generate_pseudo_legal_moves_inner(game_state, color.opposite(), true);
    let is_in_check = attack_moves.iter().any(|_move| {
      let result = _move.to == king_index;
      result
    });
    return (is_in_check, king_index)
  }
  (true, 0)
}

fn find_king(game_state: &GameState, color: Color) -> Option<usize> {
  for (index, square) in game_state.board.iter().enumerate() {
    println!("square: {:?}", square);
    if square.piece == Piece::King && square.color == color {
      return Some(index);
    }
  }
  None
}

pub fn perform_move(mut game_state: GameState, next_move: Move) -> GameState {
  let Move { from, to, .. } = next_move;
  game_state.board[to] = EMPTY_SQUARE;
  game_state.board.swap(from, to);
  if next_move.castle {
    if to == 2 {
      game_state.board.swap(0, 3);
    } else if to == 6 {
      game_state.board.swap(5, 7);
    } else if to == 58 {
      game_state.board.swap(56, 59);
    } else if to == 62 {
      game_state.board.swap(61, 63);
    }
  }
  game_state.move_list.push(next_move);
  game_state.turn = game_state.turn.opposite();

  game_state = update_castle_availability(game_state, from, to);

  game_state
}

fn update_castle_availability(mut game_state: GameState, from: usize, to: usize) -> GameState {
  let black_king_moved = from == 4;
  let black_queen_rook_moved_or_captured = from == 0 || to == 0;
  let black_king_rook_moved_or_captured = from == 7 || to == 7;
  let white_king_moved = from == 60;
  let white_queen_rook_moved_or_captured = from == 56 || to == 56;
  let white_king_rook_moved_or_captured = from == 63 || to == 63;

  if black_king_moved || black_king_rook_moved_or_captured {
    game_state.castle.black_kingside = false;
  }
  if black_king_moved || black_queen_rook_moved_or_captured {
    game_state.castle.black_queenside = false;
  }
  if white_king_moved || white_king_rook_moved_or_captured {
    game_state.castle.white_kingside = false;
  }
  if white_king_moved || white_queen_rook_moved_or_captured {
    game_state.castle.white_queenside = false;
  }

  game_state
}

// Generates pseudo legal moves, then removes the ones with the king in check.
// This is slow and should be updated later.
pub fn generate_legal_moves(game_state: &GameState) -> Vec<Move> {
  let pseudo_legal_moves = generate_pseudo_legal_moves(&game_state);
  pseudo_legal_moves.into_iter().filter(|&_move| {
    let mut game_state_clone = game_state.clone();
    game_state_clone = perform_move(game_state_clone, _move);
    let (result, _) = in_check(&game_state_clone, game_state.turn);
    !result
  }).collect()
}

pub fn generate_pseudo_legal_moves(game_state: &GameState) -> Vec<Move> {
  generate_pseudo_legal_moves_inner(game_state, game_state.turn, false)
}

fn generate_pseudo_legal_moves_inner(game_state: &GameState, color: Color, attack_only: bool) -> Vec<Move> {
  let mut moves = vec!();

  let board = &game_state.board;
  for (index, square) in board.iter().enumerate() {
    if square.color == color {
      moves = match square.piece {
        Piece::Pawn => gen_moves_pawn(game_state, color, index, attack_only, moves),
        Piece::Bishop | Piece::Rook | Piece::Queen => gen_moves_piece(&game_state, color, &square.piece, index, true, attack_only, moves),
        Piece::King | Piece::Knight => gen_moves_piece(&game_state, color, &square.piece, index, false, attack_only, moves),
        Piece::Empty => moves,
      };
      if !attack_only && square.piece == Piece::King {
        moves = gen_castle_moves(&game_state, color, moves);
      }
    }
  }

  moves
}

fn gen_moves_pawn(game_state: &GameState, color: Color, index: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
  let one_square_move_index = get_pawn_move_index(&color, index);
  if !attack_only && game_state.board[one_square_move_index].empty {
    moves.push(Move::new(index, one_square_move_index));
    if color == Color::White && index >= 48 || color == Color::Black && index < 16 {
      let two_square_move_index = get_pawn_move_index(&color, one_square_move_index);
      if game_state.board[two_square_move_index].empty {
        moves.push(Move::two_square_pawn_move(index, two_square_move_index));
      }
    }
  }

  let attack_indices = get_pawn_attack_indices(&color, index);
  for attack_index in attack_indices {
    let target_square = &game_state.board[attack_index];
    if !target_square.empty && game_state.board[attack_index].color != color {
      moves.push(Move::capture(index, attack_index, *target_square));
    }
  }

  moves
}

fn get_pawn_move_index(turn: &Color, index: usize) -> usize {
  if turn == &Color::White {
    index - 8
  } else {
    index + 8
  }
}

fn get_pawn_attack_indices(turn: &Color, index: usize) -> [usize; 2] {
  if turn == &Color::White {
    [index - 7, index - 9]
  } else {
    [index + 7, index + 9]
  }
}

fn gen_moves_piece(game_state: &GameState, color: Color, piece: &Piece, index: usize, slide: bool, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
  let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
  let mailbox_offsets = get_piece_mailbox_direction_offsets(piece);
  for mailbox_offset in mailbox_offsets {
    if slide {
      moves = gen_moves_slide_direction(game_state, color, index, mailbox_index, *mailbox_offset, attack_only, moves);
    } else {
      moves = gen_moves_hop_direction(game_state, color, index, mailbox_index, *mailbox_offset, attack_only, moves);
    }
  }
  moves
}

fn gen_moves_hop_direction(game_state: &GameState, color: Color, index: usize, mailbox_index: usize, mailbox_offset: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
  let target_mailbox_index_plus = mailbox_index + mailbox_offset;
  let target_mailbox_index_minus = mailbox_index - mailbox_offset;
  for target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
    let generated_move = gen_move_from_mailbox(game_state, color, target_mailbox_index, index);
    match generated_move {
      // Off the board
      None => (),
      Some(generated_move) => {
        if !attack_only || generated_move.capture {
          moves.push(generated_move);
        }
      },
    };
  }
  moves
}

fn gen_moves_slide_direction(game_state: &GameState, color: Color, index: usize, mailbox_index: usize, mailbox_offset: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
  let target_mailbox_index_plus = mailbox_index + mailbox_offset;
  let target_mailbox_index_minus = mailbox_index - mailbox_offset;
  for mut target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
    loop {
      let generated_move = gen_move_from_mailbox(game_state, color, target_mailbox_index, index);
      match generated_move {
        // Off the board
        None => break,
        Some(generated_move) => {
          let capture = generated_move.capture;
          if !attack_only || capture {
            moves.push(generated_move);
          }
          if capture {
            break;
          }
          if target_mailbox_index < mailbox_index {
            target_mailbox_index -= mailbox_offset;
          } else {
            target_mailbox_index += mailbox_offset;
          }
        }
      }
    }
  }
  moves
}

fn gen_castle_moves(game_state: &GameState, color: Color, mut moves: Vec<Move>) -> Vec<Move> {
  if color == Color::Black {
    if game_state.castle.black_queenside && game_state.board[1].empty && game_state.board[2].empty && game_state.board[3].empty {
      moves.push(Move::castle(4, 2));
    }
    if game_state.castle.black_kingside && game_state.board[5].empty && game_state.board[6].empty {
      moves.push(Move::castle(4, 6));
    }
  } else {
    if game_state.castle.white_queenside && game_state.board[57].empty && game_state.board[58].empty && game_state.board[59].empty {
      moves.push(Move::castle(60, 58));
    }
    if game_state.castle.white_kingside && game_state.board[61].empty && game_state.board[62].empty {
      moves.push(Move::castle(60, 62));
    }
  }
  moves
}

fn gen_move_from_mailbox(game_state: &GameState, color: Color, target_square_mailbox_index: usize, start_index: usize) -> Option<Move> {
  let target_square_index = MAILBOX[target_square_mailbox_index];
  match target_square_index {
    // Off the board
    None => None,
    Some(target_square_index) => {
      let target_square = &game_state.board[target_square_index];
      gen_move_to_index(game_state, color, target_square, start_index, target_square_index)
    }
  }
}

fn gen_move_to_index(game_state: &GameState, color: Color, target_square: &Square, from: usize, to: usize) -> Option<Move> {
  if target_square.empty {
    Some(Move::new(from, to))
  } else {
    if target_square.color != color {
      Some(Move::capture(from, to, *target_square))
    } else {
      None
    }
  }
}

fn get_piece_mailbox_direction_offsets(piece: &Piece) -> &[usize] {
  match piece {
    Piece::Bishop => &DIAGONAL_MAILBOX_DIRECTION_OFFSETS,
    Piece::Rook => &CARDINAL_MAILBOX_DIRECTION_OFFSETS,
    Piece::Queen | Piece::King => &ALL_MAILBOX_DIRECTION_OFFSETS,
    Piece::Knight => &KNIGHT_MAILBOX_DIRECTION_OFFSETS,
    // Pawn moves are calculated differently
    _ => &[],
  }
}

#[cfg(test)]
mod state_tests {
  use super::*;
  use super::fen_util::*;
  use super::types::*;

  #[test]
  fn in_check_test() {
    let game_state = get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR w KQkq -");
    assert_eq!(in_check(&game_state, Color::Black), (true, 4));
  }

  #[test]
  fn legal_moves_test() {
    let game_state = get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR w KQkq -");
    let legal_moves = generate_legal_moves(&game_state);
    println!("{:?}", legal_moves);
  }
}

#[cfg(test)]
mod perft_tests {
  use super::*;
  use super::fen_util::*;

  #[test]
  fn generate_first_move() {
    let moves = generate_pseudo_legal_moves(&GameState::default());
    assert_eq!(moves.len(), 20);
  }

  #[test]
  fn perft_pos_2() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let moves = generate_pseudo_legal_moves(&game_state);
    assert_eq!(moves.len(), 48);
  }
}
