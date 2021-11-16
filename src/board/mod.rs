pub mod types;
pub mod constants;
pub mod fen_util;

use serde::{Serialize, Deserialize};
use constants::*;
use types::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
  pub board: Board,
  pub turn: Color,
  pub castle: CastleAvailability,
  pub en_passant_index: Option<usize>,
  pub move_list: Vec<Move>,
  pub next_pseudo_legal_moves: Option<Vec<Move>>,
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      board: Default::default(),
      turn: Color::White,
      castle: Default::default(),
      en_passant_index: None,
      move_list: vec!(),
      next_pseudo_legal_moves: None,
    }
  }
}

impl GameState {
  pub fn is_opponent_in_check(&self) -> bool {
    let king_index = self.board.find_king(self.turn.opposite());
    let moves = self.generate_pseudo_legal_moves();
    if let Some(king_index) = king_index {
      return moves.iter().any(|m| m.to == king_index as usize);
    }
    true
  }

  pub fn is_in_check(&self) -> (bool, usize) {
    let king_index = self.board.find_king(self.turn);
    let moves = self.generate_pseudo_legal_moves_inner(self.turn.opposite(), false);
    if let Some(king_index) = king_index {
      let is_in_check = moves.iter().any(|m| m.to == king_index as usize);
      return (is_in_check, king_index as usize)
    }
    (true, 0)
  }

  pub fn perform_move(&mut self, next_move: Move) {
    let Move { from, to, .. } = next_move;
    self.move_list.push(next_move);

    self.board.move_from_to(from, to);
    if next_move.castle {
      if to == 2 {
        self.board.move_from_to(0, 3);
      } else if to == 6 {
        self.board.move_from_to(7, 5);
      } else if to == 58 {
        self.board.move_from_to(56, 59);
      } else if to == 62 {
        self.board.move_from_to(63, 61);
      }
    }

    self.update_castle_availability(from, to);

    if next_move.en_passant {
      let captured_pawn_index = if from > to { to + 8 } else { to - 8 };
      self.board.clear_square(captured_pawn_index);
    }

    if next_move.two_square_pawn_move {
      self.en_passant_index = Some((from + to) / 2);
    } else {
      self.en_passant_index = None;
    }

    if let Some(promotion_piece) = next_move.promotion_piece {
      self.board.update_square(to, self.turn, promotion_piece);
    }

    self.turn = self.turn.opposite();
    self.next_pseudo_legal_moves = None;
  }

  fn update_castle_availability(&mut self, from: usize, to: usize) {
    let black_king_moved = from == 4;
    let black_queen_rook_moved_or_captured = from == 0 || to == 0;
    let black_king_rook_moved_or_captured = from == 7 || to == 7;
    let white_king_moved = from == 60;
    let white_queen_rook_moved_or_captured = from == 56 || to == 56;
    let white_king_rook_moved_or_captured = from == 63 || to == 63;

    if black_king_moved || black_king_rook_moved_or_captured {
      self.castle.black_kingside = false;
    }
    if black_king_moved || black_queen_rook_moved_or_captured {
      self.castle.black_queenside = false;
    }
    if white_king_moved || white_king_rook_moved_or_captured {
      self.castle.white_kingside = false;
    }
    if white_king_moved || white_queen_rook_moved_or_captured {
      self.castle.white_queenside = false;
    }
  }

  // Generates pseudo legal moves, then removes the ones with the king in check.
  // This is slow and should be updated later.
  pub fn generate_legal_moves(&self) -> Vec<GameState> {
    let pseudo_legal_moves = self.generate_pseudo_legal_moves();
    let (current_is_in_check, _) = self.is_in_check();
    pseudo_legal_moves.iter().filter_map(|&_move| {
      if _move.castle && current_is_in_check {
        return None;
      }
      let mut game_state_clone = self.clone();
      game_state_clone.perform_move(_move);
      let moves = game_state_clone.generate_pseudo_legal_moves();
      if _move.castle {
        let check_index = (_move.from + _move.to) / 2;
        let castle_into_or_through_check = moves.iter().any(|next_move| [_move.to, check_index].contains(&next_move.to));
        if castle_into_or_through_check {
          return None;
        } else {
          game_state_clone.next_pseudo_legal_moves = Some(moves);
          return Some(game_state_clone);
        }
      } else {
        let is_in_check = game_state_clone.is_opponent_in_check();
        if is_in_check {
          None
        } else {
          game_state_clone.next_pseudo_legal_moves = Some(moves);
          Some(game_state_clone)
        }
      }
    }).collect()
  }

  pub fn generate_legal_moves_at_depth(&self, depth: usize) -> Vec<GameState> {
    let mut game_states = self.generate_legal_moves();

    let mut curr_depth = 1;
    while curr_depth < depth {
      curr_depth += 1;
      game_states = game_states.iter_mut().fold(vec!(), |mut next, game_state| {
        let mut next_move_list = game_state.generate_legal_moves();
        next.append(&mut next_move_list);
        next
      });
    }

    game_states
  }

  pub fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
    if let Some(next_pseudo_legal_moves) = &self.next_pseudo_legal_moves {
      next_pseudo_legal_moves.clone()
    } else {
      self.generate_pseudo_legal_moves_inner(self.turn, false)
    }
  }

  fn generate_pseudo_legal_moves_inner(&self, color: Color, attack_only: bool) -> Vec<Move> {
    let mut moves = vec!();

    let board = &self.board;
    for index in 0..64 {
      let (square_color, square_piece) = board.get_square(index);
      if square_color == color {
        moves = match square_piece {
          Piece::Pawn => self.gen_moves_pawn(color, index, attack_only, moves),
          Piece::Bishop | Piece::Rook | Piece::Queen => self.gen_moves_piece(color, &square_piece, index, true, attack_only, moves),
          Piece::King | Piece::Knight => self.gen_moves_piece(color, &square_piece, index, false, attack_only, moves),
          Piece::Empty => moves,
        };
        if !attack_only && square_piece == Piece::King {
          moves = self.gen_castle_moves(color, moves);
        }
      }
    }

    moves
  }

  fn gen_moves_pawn(&self, color: Color, index: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
    let one_square_move_index = get_pawn_move_index(&color, index);
    if !attack_only && self.board.is_index_empty(one_square_move_index) {
      if color == Color::White && one_square_move_index < 8 || color == Color::Black && one_square_move_index > 55 {
        [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen].iter().for_each(|piece| {
          moves.push(Move::promotion(index, one_square_move_index, *piece));
        });
      } else {
        moves.push(Move::new(index, one_square_move_index));
        if color == Color::White && index >= 48 || color == Color::Black && index < 16 {
          let two_square_move_index = get_pawn_move_index(&color, one_square_move_index);
          if self.board.is_index_empty(two_square_move_index) {
            moves.push(Move::two_square_pawn_move(index, two_square_move_index));
          }
        }
      }
    }

    let mailbox_attack_indices = get_pawn_mailbox_attack_indices(&color, index);
    let en_passant_index = self.en_passant_index.unwrap_or(100);
    for mailbox_attack_index in mailbox_attack_indices {
      if let Some(attack_index) = MAILBOX[mailbox_attack_index] {
        if attack_index == en_passant_index {
          moves.push(Move::en_passant(index, attack_index));
        } else {
          let is_target_empty = &self.board.is_index_empty(attack_index);
          if !is_target_empty && !self.board.is_index_of_color(attack_index, color) {
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

  fn gen_moves_piece(&self, color: Color, piece: &Piece, index: usize, slide: bool, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
    let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
    let mailbox_offsets = get_piece_mailbox_direction_offsets(piece);
    for mailbox_offset in mailbox_offsets {
      if slide {
        moves = self.gen_moves_slide_direction(color, index, mailbox_index, *mailbox_offset, attack_only, moves);
      } else {
        moves = self.gen_moves_hop_direction(color, index, mailbox_index, *mailbox_offset, attack_only, moves);
      }
    }
    moves
  }

  fn gen_moves_hop_direction(&self, color: Color, index: usize, mailbox_index: usize, mailbox_offset: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
    let target_mailbox_index_plus = mailbox_index + mailbox_offset;
    let target_mailbox_index_minus = mailbox_index - mailbox_offset;
    for target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
      let generated_move = self.gen_move_from_mailbox(color, target_mailbox_index, index);
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

  fn gen_moves_slide_direction(&self, color: Color, index: usize, mailbox_index: usize, mailbox_offset: usize, attack_only: bool, mut moves: Vec<Move>) -> Vec<Move> {
    let target_mailbox_index_plus = mailbox_index + mailbox_offset;
    let target_mailbox_index_minus = mailbox_index - mailbox_offset;
    for mut target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
      loop {
        let generated_move = self.gen_move_from_mailbox(color, target_mailbox_index, index);
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

  fn gen_castle_moves(&self, color: Color, mut moves: Vec<Move>) -> Vec<Move> {
    if color == Color::Black {
      if self.castle.black_queenside && self.board.castle_black_queenside_open() {
        moves.push(Move::castle(4, 2));
      }
      if self.castle.black_kingside && self.board.castle_black_kingside_open() {
        moves.push(Move::castle(4, 6));
      }
    } else {
      if self.castle.white_queenside && self.board.castle_white_queenside_open() {
        moves.push(Move::castle(60, 58));
      }
      if self.castle.white_kingside && self.board.castle_white_kingside_open() {
        moves.push(Move::castle(60, 62));
      }
    }
    moves
  }

  fn gen_move_from_mailbox(&self, color: Color, target_square_mailbox_index: usize, start_index: usize) -> Option<Move> {
    let target_square_index = MAILBOX[target_square_mailbox_index];
    match target_square_index {
      // Off the board
      None => None,
      Some(target_square_index) => {
        self.gen_move_to_index(color, start_index, target_square_index)
      }
    }
  }

  fn gen_move_to_index(&self, color: Color, from: usize, to: usize) -> Option<Move> {
    if self.board.is_index_empty(to) {
      Some(Move::new(from, to))
    } else if !self.board.is_index_of_color(to, color) {
      Some(Move::capture(from, to))
    } else {
      None
    }
  }
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
  use super::fen_util::*;

  #[test]
  fn in_check_test() {
    let game_state = get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR b KQkq -");
    assert_eq!(game_state.is_in_check(), (true, 4));
    assert_eq!(game_state.is_opponent_in_check(), false);
  }
}

#[cfg(test)]
mod perft_tests {
  use std::collections::HashMap;
  use super::*;
  use super::fen_util::*;

  #[test]
  fn perft_pos_1_depth_1() {
    let moves = GameState::default().generate_legal_moves();
    assert_eq!(moves.len(), 20);
  }

  #[test]
  fn perft_pos_1_depth_2() {
    let game_states = GameState::default().generate_legal_moves_at_depth(2);
    assert_eq!(game_states.len(), 400);
  }

  #[test]
  fn perft_pos_1_depth_3() {
    let game_states = GameState::default().generate_legal_moves_at_depth(3);
    assert_eq!(game_states.len(), 8902);
  }

  #[test]
  fn perft_pos_2_depth_1() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let moves = game_state.generate_legal_moves();
    assert_eq!(moves.len(), 48);
  }

  #[test]
  fn perft_pos_2_depth_2() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let game_states = game_state.generate_legal_moves_at_depth(2);
    assert_eq!(game_states.len(), 2039);
  }

  #[test]
  fn perft_pos_2_depth_3() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let game_states = game_state.generate_legal_moves_at_depth(3);
    assert_eq!(game_states.len(), 97862);
  }

  #[test]
  fn perft_pos_3_depth_1() {
    let game_state = get_game_state_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let moves = game_state.generate_legal_moves();
    assert_eq!(moves.len(), 14);
  }

  #[test]
  fn perft_pos_3_depth_2() {
    let game_state = get_game_state_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let game_states = game_state.generate_legal_moves_at_depth(2);
    assert_eq!(game_states.len(), 191);
  }

  #[test]
  fn perft_pos_3_depth_3() {
    let game_state = get_game_state_from_fen("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -");
    let game_states = game_state.generate_legal_moves_at_depth(3);
    assert_eq!(game_states.len(), 2812);
  }

  #[test]
  fn perft_pos_4_depth_1() {
    let game_state = get_game_state_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let moves = game_state.generate_legal_moves();
    assert_eq!(moves.len(), 6);
  }

  #[test]
  fn perft_pos_4_depth_2() {
    let game_state = get_game_state_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let game_states = game_state.generate_legal_moves_at_depth(2);
    assert_eq!(game_states.len(), 264);
  }

  #[test]
  fn perft_pos_4_depth_3() {
    let game_state = get_game_state_from_fen("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1");
    let game_states = game_state.generate_legal_moves_at_depth(3);
    assert_eq!(game_states.len(), 9467);
  }

  #[test]
  fn perft_pos_5_depth_1() {
    let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let moves = game_state.generate_legal_moves();
    assert_eq!(moves.len(), 44);
  }

  #[test]
  fn perft_pos_5_depth_2() {
    let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let game_states = game_state.generate_legal_moves_at_depth(2);
    assert_eq!(game_states.len(), 1486);
  }

  #[test]
  fn perft_pos_5_depth_3() {
    let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
    let game_states = game_state.generate_legal_moves_at_depth(3);
    let move_map = generate_move_map(&game_states);
    println!("{:#?}", move_map);
    assert_eq!(game_states.len(), 62379);
  }

  // passes, but slow
  // #[test]
  // fn perft_pos_5_depth_4() {
  //   let game_state = get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
  //   let game_states = game_state.generate_legal_moves_at_depth(4);
  //   assert_eq!(game_states.len(), 2_103_487 );
  // }

  #[test]
  fn perft_pos_6_depth_1() {
    let game_state = get_game_state_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let moves = game_state.generate_legal_moves();
    assert_eq!(moves.len(), 46);
  }

  #[test]
  fn perft_pos_6_depth_2() {
    let game_state = get_game_state_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let game_states = game_state.generate_legal_moves_at_depth(2);
    assert_eq!(game_states.len(), 2079);
  }

  #[test]
  fn perft_pos_6_depth_3() {
    let game_state = get_game_state_from_fen("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10");
    let game_states = game_state.generate_legal_moves_at_depth(3);
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

#[cfg(test)]
mod benchmark_tests {
  extern crate test;

  use super::fen_util::*;
  use test::Bencher;

  #[bench]
  fn perft_pos_2_pseudo_bench(b: &mut Bencher) {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    b.iter(|| game_state.generate_pseudo_legal_moves());
  }

  #[bench]
  fn perft_pos_2_legal_bench(b: &mut Bencher) {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    b.iter(|| game_state.generate_legal_moves());
  }
}
