pub mod types;
pub mod constants;
pub mod fen_util;

use constants::*;
use types::{Color, GameState, Move, Piece, Square};

const CARDINAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [1, 10];
const DIAGONAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [9, 11];
const ALL_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [1, 9, 10, 11];
const KNIGHT_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [8, 12, 19, 21];

pub fn in_check(game_state: &GameState) -> bool {
  let king = find_king(game_state);
  false
}

fn find_king(game_state: &GameState) -> usize {
  for (index, square) in game_state.board.iter().enumerate() {
    if square.piece == Piece::King && square.color == game_state.turn {
      return index;
    }
  }
  unreachable!()
}

pub fn perform_move(mut game_state: GameState, next_move: Move) -> GameState {
  let Move { from, to, .. } = next_move;
  game_state.board[to] = EMPTY_SQUARE;
  game_state.board.swap(from, to);
  game_state.move_list.push(next_move);
  game_state.turn = match game_state.turn {
    Color::White => Color::Black,
    Color::Black => Color::White,
    Color::Empty => unreachable!(),
  };
  game_state
}

pub fn undo_last_move(mut game_state: GameState, mut move_list: Vec<Move>) -> (GameState, Vec<Move>) {
  if let Some(last_move) = move_list.pop() {
    let Move { from, to, capture, captured_square, .. } = last_move;
    if capture {
      game_state.board[from] = captured_square;
    }
    game_state.board.swap(from, to);
  }
  (game_state, move_list)
}

pub fn generate_pseudo_legal_moves(game_state: &GameState) -> Vec<Move> {
  let mut moves = vec!();

  let board = &game_state.board;
  for (index, square) in board.iter().enumerate() {
    if square.color == game_state.turn {
      moves = match square.piece {
        Piece::Pawn => gen_moves_pawn(game_state, index, moves),
        Piece::Bishop | Piece::Rook | Piece::Queen => gen_moves_piece(&game_state, &square.piece, index, true, moves),
        Piece::King | Piece::Knight => gen_moves_piece(&game_state, &square.piece, index, false, moves),
        Piece::Empty => moves,
      };
    }
  }

  return moves;
}

fn gen_moves_pawn(game_state: &GameState, index: usize, mut moves: Vec<Move>) -> Vec<Move> {
  let one_square_move_index = get_pawn_move_index(&game_state.turn, index);
  if game_state.board[one_square_move_index].empty {
    moves.push(Move::new(index, one_square_move_index));
    if &game_state.turn == &Color::White && index >= 48 || &game_state.turn == &Color::Black && index < 16 {
      let two_square_move_index = get_pawn_move_index(&game_state.turn, one_square_move_index);
      if game_state.board[two_square_move_index].empty {
        moves.push(Move::two_square_pawn_move(index, two_square_move_index));
      }
    }
  }

  let attack_indices = get_pawn_attack_indices(&game_state.turn, index);
  for index in attack_indices {
    let target_square = &game_state.board[index];
    if !target_square.empty && game_state.board[index].color != game_state.turn {
      moves.push(Move::capture(index, index, *target_square));
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

fn gen_moves_piece(game_state: &GameState, piece: &Piece, index: usize, slide: bool, mut moves: Vec<Move>) -> Vec<Move> {
  let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
  let mailbox_offsets = get_piece_mailbox_direction_offsets(piece);
  for mailbox_offset in mailbox_offsets {
    if slide {
      moves = gen_moves_slide_direction(game_state, index, mailbox_index, *mailbox_offset, moves);
    } else {
      moves = gen_moves_hop_direction(game_state, index, mailbox_index, *mailbox_offset, moves);
    }
  }
  moves
}

fn gen_moves_hop_direction(game_state: &GameState, index: usize, mailbox_index: usize, mailbox_offset: usize, mut moves: Vec<Move>) -> Vec<Move> {
  let target_mailbox_index_plus = mailbox_index + mailbox_offset;
  let target_mailbox_index_minus = mailbox_index - mailbox_offset;
  for target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
    let generated_move = gen_move_from_mailbox(game_state, target_mailbox_index, index);
    match generated_move {
      // Off the board
      None => (),
      Some(generated_move) => {
        moves.push(generated_move);
      },
    };
  }
  moves
}

fn gen_moves_slide_direction(game_state: &GameState, index: usize, mailbox_index: usize, mailbox_offset: usize, mut moves: Vec<Move>) -> Vec<Move> {
  let target_mailbox_index_plus = mailbox_index + mailbox_offset;
  let target_mailbox_index_minus = mailbox_index - mailbox_offset;
  for mut target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
    loop {
      let generated_move = gen_move_from_mailbox(game_state, target_mailbox_index, index);
      match generated_move {
        // Off the board
        None => break,
        Some(generated_move) => {
          let capture = generated_move.capture;
          moves.push(generated_move);
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

fn gen_move_from_mailbox(game_state: &GameState, target_square_mailbox_index: usize, start_index: usize) -> Option<Move> {
  let target_square_index = MAILBOX[target_square_mailbox_index];
  match target_square_index {
    // Off the board
    None => None,
    Some(target_square_index) => {
      let target_square = &game_state.board[target_square_index];
      gen_move_to_index(game_state, target_square, start_index, target_square_index)
    }
  }
}

fn gen_move_to_index(game_state: &GameState, target_square: &Square, from: usize, to: usize) -> Option<Move> {
  if target_square.empty {
    Some(Move::new(from, to))
  } else {
    if target_square.color != game_state.turn {
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
mod fen_tests {
  use super::fen_util::*;
  use crate::board_to_fen_string;

  #[test]
  fn board_to_fen() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let fen = board_to_fen_string(game_state.board);

    assert_eq!(fen, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
  }
}

#[cfg(test)]
mod perft_tests {
  use super::*;
  use super::fen_util::*;
  use crate::board::constants::INITIAL_GAME_STATE;

  #[test]
  fn generate_first_move() {
    let moves = generate_pseudo_legal_moves(&INITIAL_GAME_STATE);
    assert_eq!(moves.len(), 20);
  }

  #[test]
  fn perft_pos_2() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let moves = generate_pseudo_legal_moves(&game_state);
    for m in &moves {
      let from_square = get_square_from_index(m.from);
      let to_square = get_square_from_index(m.to);
      println!("{}", from_square + &to_square);
    }
    assert_eq!(moves.len(), 48);
  }
}
