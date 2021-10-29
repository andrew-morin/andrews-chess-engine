pub mod types;
pub mod constants;

use constants::{MAILBOX};
use types::{Board, Color, GameState, Move, Piece, Square};

const CARDINAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [1, 10];
const DIAGONAL_MAILBOX_DIRECTION_OFFSETS: [usize; 2] = [9, 11];
const ALL_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [1, 9, 10, 11];
const KNIGHT_MAILBOX_DIRECTION_OFFSETS: [usize; 4] = [8, 12, 19, 21];

pub fn generate_pseudo_legal_moves(game_state: &GameState) -> Vec<Move> {
  let mut moves = vec!();

  let board = game_state.board;
  for (index, square) in board.iter().enumerate() {
    let genereated_moves = match square.piece {
      Piece::Bishop | Piece::Rook | Piece::Queen => gen_moves_slide(&game_state, index, square, moves),
    };
  }

  return moves;
}

fn gen_moves_slide(game_state: &GameState, index: usize, square: &Square, mut moves: Vec<Move>) {
  let mailbox_offsets = get_piece_mailbox_direction_offsets(&square.piece);
  for mailbox_offset in mailbox_offsets {
    let target_square_mailbox_index = index + mailbox_offset;
    loop {
      let target_square_index = MAILBOX[target_square_mailbox_index];
      match target_square_index {
        // Off the board
        None => break,
        Some(target_square_index) => {
          let target_square = &game_state.board[target_square_index];
          let generated_move = gen_move_to_index(game_state, target_square, index, target_square_index);
          if let Some(move_to_add) = generated_move {
            moves.push(move_to_add);
            target_square_mailbox_index += mailbox_offset;
          } else {
            // Square occupied by friendly piece
            break;
          }
        }
      }
    }
  }
}

fn gen_move_to_index(game_state: &GameState, target_square: &Square, from: usize, to: usize) -> Option<Move> {
  if target_square.empty {
    Some(Move::new(from, to))
  } else {
    if target_square.color != game_state.turn {
      Some(Move::capture(from, to))
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
