pub mod types;
pub mod constants;
pub mod fen_util;

use constants::*;
use types::*;

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
  for index in 0..64 {
    if game_state.board.is_index_of_color_and_piece(index, color, Piece::King) {
      return Some(index);
    }
  }
  None
}

pub fn perform_move(mut game_state: GameState, next_move: Move) -> GameState {
  let Move { from, to, .. } = next_move;
  let (from_color, from_piece) = game_state.board.get_square(from);
  game_state.board.update_square(to, from_color, from_piece);
  game_state.board.clear_square(from);
  if next_move.castle {
    if to == 2 {
      game_state.board.update_square(3, from_color, Piece::Rook);
      game_state.board.clear_square(0);
    } else if to == 6 {
      game_state.board.update_square(5, from_color, Piece::Rook);
      game_state.board.clear_square(7);
    } else if to == 58 {
      game_state.board.update_square(59, from_color, Piece::Rook);
      game_state.board.clear_square(56);
    } else if to == 62 {
      game_state.board.update_square(61, from_color, Piece::Rook);
      game_state.board.clear_square(63);
    }
  }
  game_state.move_list.push(next_move);

  game_state = update_castle_availability(game_state, from, to);

  if next_move.en_passant {
    let captured_pawn_index = if from > to { to + 8 } else { to - 8 };
    game_state.board.clear_square(captured_pawn_index);
  }

  if next_move.two_square_pawn_move {
    game_state.en_passant_index = Some((from + to) / 2);
  } else {
    game_state.en_passant_index = None;
  }

  if let Some(promotion_piece) = next_move.promotion_piece {
    game_state.board.update_square(to, game_state.turn, promotion_piece);
  }

  game_state.turn = game_state.turn.opposite();
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
  let (current_is_in_check, _) = in_check(&game_state, game_state.turn);
  pseudo_legal_moves.into_iter().filter(|&_move| {
    if _move.castle && current_is_in_check {
      return false;
    }
    let mut game_state_clone = game_state.clone();
    game_state_clone = perform_move(game_state_clone, _move);
    let attack_moves = generate_pseudo_legal_moves_inner(&game_state_clone, game_state_clone.turn, true);
    if _move.castle {
      let check_index = (_move.from + _move.to) / 2;
      let castle_into_or_through_check = attack_moves.iter().any(|attack| [_move.to, check_index].contains(&attack.to));
      return !castle_into_or_through_check;
    }
    let (is_in_check, _) = in_check(&game_state_clone, game_state_clone.turn.opposite());
    !is_in_check
  }).collect()
}

pub fn generate_legal_moves_at_depth(game_state: &GameState, depth: usize) -> Vec<GameState> {
  let moves = generate_legal_moves(&game_state);
  let mut game_states: Vec<GameState> = moves.iter().map(|_move| perform_move(game_state.clone(), *_move)).collect::<Vec<GameState>>();

  let mut curr_depth = 1;
  while curr_depth < depth {
    curr_depth += 1;
    let mut next: Vec<GameState> = vec!();
    game_states.iter().for_each(|game_state| {
      let next_move_list = generate_legal_moves(&game_state);
      for next_move in next_move_list.iter() {
        let new_game_state = perform_move(game_state.clone(), *next_move);
        next.push(new_game_state);
      }
    });
    game_states = next;
  }

  game_states
}

pub fn generate_pseudo_legal_moves(game_state: &GameState) -> Vec<Move> {
  generate_pseudo_legal_moves_inner(game_state, game_state.turn, false)
}

fn generate_pseudo_legal_moves_inner(game_state: &GameState, color: Color, attack_only: bool) -> Vec<Move> {
  let mut moves = vec!();

  let board = &game_state.board;
  for index in 0..64 {
    let (square_color, square_piece) = board.get_square(index);
    if square_color == color {
      moves = match square_piece {
        Piece::Pawn => gen_moves_pawn(game_state, color, index, attack_only, moves),
        Piece::Bishop | Piece::Rook | Piece::Queen => gen_moves_piece(&game_state, color, &square_piece, index, true, attack_only, moves),
        Piece::King | Piece::Knight => gen_moves_piece(&game_state, color, &square_piece, index, false, attack_only, moves),
        Piece::Empty => moves,
      };
      if !attack_only && square_piece == Piece::King {
        moves = gen_castle_moves(&game_state, color, moves);
      }
    }
  }

  moves
}

fn gen_moves_pawn(game_state: &GameState, color: Color, index: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
  let one_square_move_index = get_pawn_move_index(&color, index);
  if !attack_only && game_state.board.is_index_empty(one_square_move_index) {
    if color == Color::White && one_square_move_index < 8 || color == Color::Black && one_square_move_index > 55 {
      [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen].iter().for_each(|piece| {
        moves.push(Move::promotion(index, one_square_move_index, *piece));
      });
    } else {
      moves.push(Move::new(index, one_square_move_index));
      if color == Color::White && index >= 48 || color == Color::Black && index < 16 {
        let two_square_move_index = get_pawn_move_index(&color, one_square_move_index);
        if game_state.board.is_index_empty(two_square_move_index) {
          moves.push(Move::two_square_pawn_move(index, two_square_move_index));
        }
      }
    }
  }

  let mailbox_attack_indices = get_pawn_mailbox_attack_indices(&color, index);
  let en_passant_index = game_state.en_passant_index.unwrap_or(100);
  for mailbox_attack_index in mailbox_attack_indices {
    if let Some(attack_index) = MAILBOX[mailbox_attack_index] {
      if attack_index == en_passant_index {
        moves.push(Move::en_passant(index, attack_index));
      } else {
        let is_target_empty = &game_state.board.is_index_empty(attack_index);
        if !is_target_empty && !game_state.board.is_index_of_color(attack_index, color) {
          if color == Color::White && attack_index < 8 || color == Color::Black && attack_index > 55 {
            [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen].iter().for_each(|piece| {
              moves.push(Move::promotion_capture(index, attack_index, *piece));
            });
          } else {
            moves.push(Move::capture(index, attack_index));
          }
        }
      }
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

fn get_pawn_mailbox_attack_indices(turn: &Color, index: usize) -> [usize; 2] {
  let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
  if turn == &Color::White {
    [mailbox_index - 9, mailbox_index - 11]
  } else {
    [mailbox_index + 9, mailbox_index + 11]
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
    if game_state.castle.black_queenside && game_state.board.castle_black_queenside_open() {
      moves.push(Move::castle(4, 2));
    }
    if game_state.castle.black_kingside && game_state.board.castle_black_kingside_open() {
      moves.push(Move::castle(4, 6));
    }
  } else {
    if game_state.castle.white_queenside && game_state.board.castle_white_queenside_open() {
      moves.push(Move::castle(60, 58));
    }
    if game_state.castle.white_kingside && game_state.board.castle_white_kingside_open() {
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
      gen_move_to_index(game_state, color, start_index, target_square_index)
    }
  }
}

fn gen_move_to_index(game_state: &GameState, color: Color, from: usize, to: usize) -> Option<Move> {
  if game_state.board.is_index_empty(to) {
    Some(Move::new(from, to))
  } else if !game_state.board.is_index_of_color(to, color) {
    Some(Move::capture(from, to))
  } else {
    None
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

  #[test]
  fn in_check_test() {
    let game_state = get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR w KQkq -");
    assert_eq!(in_check(&game_state, Color::Black), (true, 4));
  }
}

#[cfg(test)]
mod perft_tests {
  use std::collections::HashMap;
  use super::*;
  use super::fen_util::*;

  #[test]
  fn perft_pos_1_depth_1() {
    let moves = generate_legal_moves(&GameState::default());
    assert_eq!(moves.len(), 20);
  }

  #[test]
  fn perft_pos_1_depth_2() {
    let game_states = generate_legal_moves_at_depth(&GameState::default(), 2);
    assert_eq!(game_states.len(), 400);
  }

  #[test]
  fn perft_pos_1_depth_3() {
    let game_states = generate_legal_moves_at_depth(&GameState::default(), 3);
    assert_eq!(game_states.len(), 8902);
  }

  #[test]
  fn perft_pos_2_depth_1() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let moves = generate_legal_moves(&game_state);
    assert_eq!(moves.len(), 48);
  }

  #[test]
  fn perft_pos_2_depth_2() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let game_states = generate_legal_moves_at_depth(&game_state, 2);
    assert_eq!(game_states.len(), 2039);
  }

  #[test]
  fn perft_pos_2_depth_3() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let game_states = generate_legal_moves_at_depth(&game_state, 3);
    assert_eq!(game_states.len(), 97862);
  }

  #[test]
  fn perft_pos_3_depth_1() {
    let game_state = get_game_state_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let moves = generate_legal_moves(&game_state);
    assert_eq!(moves.len(), 14);
  }

  #[test]
  fn perft_pos_3_depth_2() {
    let game_state = get_game_state_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let game_states = generate_legal_moves_at_depth(&game_state, 2);
    assert_eq!(game_states.len(), 191);
  }

  #[test]
  fn perft_pos_3_depth_3() {
    let game_state = get_game_state_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let game_states = generate_legal_moves_at_depth(&game_state, 3);
    assert_eq!(game_states.len(), 2812);
  }

  #[test]
  fn perft_pos_4_depth_1() {
    let game_state = get_game_state_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let moves = generate_legal_moves(&game_state);
    assert_eq!(moves.len(), 6);
  }

  #[test]
  fn perft_pos_4_depth_2() {
    let game_state = get_game_state_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let game_states = generate_legal_moves_at_depth(&game_state, 2);
    assert_eq!(game_states.len(), 264);
  }

  #[test]
  fn perft_pos_4_depth_3() {
    let game_state = get_game_state_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let game_states = generate_legal_moves_at_depth(&game_state, 3);
    assert_eq!(game_states.len(), 9467);
  }

  #[test]
  fn perft_pos_5_depth_1() {
    let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let moves = generate_legal_moves(&game_state);
    assert_eq!(moves.len(), 44);
  }

  #[test]
  fn perft_pos_5_depth_2() {
    let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let game_states = generate_legal_moves_at_depth(&game_state, 2);
    assert_eq!(game_states.len(), 1486);
  }

  #[test]
  fn perft_pos_5_depth_3() {
    let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let game_states = generate_legal_moves_at_depth(&game_state, 3);
    assert_eq!(game_states.len(), 62379);
  }

  // passes, but slow
  // #[test]
  // fn perft_pos_5_depth_4() {
  //   let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
  //   let game_states = generate_legal_moves_at_depth(&game_state, 4);
  //   assert_eq!(game_states.len(), 2_103_487 );
  // }

  #[test]
  fn perft_pos_6_depth_1() {
    let game_state = get_game_state_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let moves = generate_legal_moves(&game_state);
    assert_eq!(moves.len(), 46);
  }

  #[test]
  fn perft_pos_6_depth_2() {
    let game_state = get_game_state_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let game_states = generate_legal_moves_at_depth(&game_state, 2);
    assert_eq!(game_states.len(), 2079);
  }

  #[test]
  fn perft_pos_6_depth_3() {
    let game_state = get_game_state_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let game_states = generate_legal_moves_at_depth(&game_state, 3);
    assert_eq!(game_states.len(), 89890);
  }

  fn generate_move_map(game_states: &Vec<GameState>) -> HashMap<String, usize> {
    let mut move_map: HashMap<String, usize> = HashMap::new();
    game_states.iter().for_each(|game_state| {
      let first_move = game_state.move_list.get(0);
      if let Some(first_move) = first_move {
        let from_square = get_square_from_index(first_move.from);
        let to_square = get_square_from_index(first_move.to);
        let key = from_square + &to_square;
        move_map.entry(key)
          .and_modify(|e| { *e += 1 })
          .or_insert(1);
      }
    });
    move_map
  }
}
