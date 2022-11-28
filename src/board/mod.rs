pub mod constants;
pub mod fen_util;
pub mod types;

use constants::*;
use serde::{Deserialize, Serialize};
use types::*;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
    pub board: Board,
    pub turn: Color,
    pub castle: CastleAvailability,
    pub en_passant_index: Option<u32>,
    pub move_list: Vec<Move>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            board: Default::default(),
            turn: Color::White,
            castle: Default::default(),
            en_passant_index: None,
            move_list: vec![],
        }
    }
}

#[derive(Debug, Default, PartialEq)]
struct AttackData {
    pin_ray_bitmask: u64,
    opponent_sliding_attack_bitmask: u64,
    opponent_knight_attack_bitmask: u64,
    opponent_pawn_attack_bitmask: u64,
    opponent_attack_bitmask: u64,
    check_ray_bitmask: u64,
    in_check: bool,
    in_double_check: bool,
    pin_exists_in_position: bool,
}

impl GameState {
    pub fn is_opponent_in_check(&self) -> bool {
        let (is_in_check, _) = self.is_in_check_inner(self.turn.opposite());
        is_in_check
    }

    pub fn is_in_check(&self) -> (bool, u32) {
        self.is_in_check_inner(self.turn)
    }

    fn is_in_check_inner(&self, color: Color) -> (bool, u32) {
        let king_index = self.board.find_king(color);
        (self.board.is_index_under_attack(king_index), king_index)
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
    }

    fn update_castle_availability(&mut self, from: u32, to: u32) {
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
    // TODO: This is slow and should be updated later.
    pub fn generate_legal_moves(&self) -> Vec<GameState> {
        let pseudo_legal_moves = self.generate_pseudo_legal_moves();
        let (current_is_in_check, _) = self.is_in_check();
        pseudo_legal_moves
            .iter()
            .filter(|&next_move| !next_move.castle || !current_is_in_check)
            .filter_map(|&next_move| {
                let mut game_state_clone = self.clone();
                game_state_clone.perform_move(next_move);
                if next_move.castle {
                    let castle_into_check = game_state_clone.is_opponent_in_check();
                    let check_index = (next_move.from + next_move.to) / 2;
                    let castle_through_check =
                        game_state_clone.board.is_index_under_attack(check_index);
                    if castle_into_check || castle_through_check {
                        None
                    } else {
                        Some(game_state_clone)
                    }
                } else {
                    let is_in_check = game_state_clone.is_opponent_in_check();
                    if is_in_check {
                        None
                    } else {
                        Some(game_state_clone)
                    }
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn generate_legal_moves_new(&self) -> Vec<GameState> {
        let _attack_data = self.generate_attack_data();

        vec![]
    }

    fn generate_attack_data(&self) -> AttackData {
        let friendly_king_bit =
            self.board.get_color_bitmask(self.turn) & self.board.get_piece_bitmask(Piece::King);
        let friendly_king_index = friendly_king_bit.trailing_zeros();
        let mut attack_data = AttackData::default();
        let opp_color = self.turn.opposite();
        let opp_bits = self.board.get_color_bitmask(opp_color);
        let opp_queen_bits = opp_bits & self.board.get_piece_bitmask(Piece::Queen);
        let opp_rook_bits = opp_bits & self.board.get_piece_bitmask(Piece::Rook);
        let opp_bishop_bits = opp_bits & self.board.get_piece_bitmask(Piece::Bishop);
        let opp_queen_rook_indices = bits_to_indices(opp_queen_bits | opp_rook_bits);
        let opp_queen_bishop_indices = bits_to_indices(opp_queen_bits | opp_bishop_bits);

        attack_data.opponent_sliding_attack_bitmask |= self.get_sliding_attack_data(
            &opp_queen_rook_indices,
            CARDINAL_MAILBOX_DIRECTION_OFFSETS_ALL,
            friendly_king_index,
        );
        attack_data.opponent_sliding_attack_bitmask |= self.get_sliding_attack_data(
            &opp_queen_bishop_indices,
            DIAGONAL_MAILBOX_DIRECTION_OFFSETS_ALL,
            friendly_king_index,
        );

        if !opp_queen_rook_indices.is_empty() {
            self.check_pins_and_checks(
                CARDINAL_MAILBOX_DIRECTION_OFFSETS_ALL,
                false,
                friendly_king_index,
                &mut attack_data,
            );
        }
        if !attack_data.in_double_check && !opp_queen_bishop_indices.is_empty() {
            self.check_pins_and_checks(
                DIAGONAL_MAILBOX_DIRECTION_OFFSETS_ALL,
                true,
                friendly_king_index,
                &mut attack_data,
            );
        }

        let opp_knight_bits = opp_bits & self.board.get_piece_bitmask(Piece::Knight);
        let opp_knight_indices = bits_to_indices(opp_knight_bits);
        let mut is_knight_check = false;

        for index in opp_knight_indices {
            let attack_bitmask = KNIGHT_ATTACK_BITMASKS[index as usize];
            attack_data.opponent_knight_attack_bitmask |= attack_bitmask;
            if !is_knight_check && friendly_king_bit & attack_bitmask != 0 {
                is_knight_check = true;
                // If already in check, then this is a double check
                attack_data.in_double_check = attack_data.in_check;
                attack_data.in_check = true;
                attack_data.check_ray_bitmask |= 1 << index;
            }
        }

        let opp_pawn_bits = opp_bits & self.board.get_piece_bitmask(Piece::Pawn);
        let opp_pawn_indices = bits_to_indices(opp_pawn_bits);
        let mut is_pawn_check = false;

        for index in opp_pawn_indices {
            let attack_mb_indices = get_pawn_mailbox_attack_indices(&self.turn.opposite(), index);
            for attack_mb_index in attack_mb_indices {
                let attack_index = MAILBOX[attack_mb_index as usize];
                if let Some(attack_index) = attack_index {
                    let attack_bitmask = 1 << attack_index;
                    attack_data.opponent_pawn_attack_bitmask |= attack_bitmask;
                    if !is_pawn_check && friendly_king_bit & attack_bitmask != 0 {
                        is_pawn_check = true;
                        // If already in check, then this is a double check
                        attack_data.in_double_check = attack_data.in_check;
                        attack_data.in_check = true;
                        attack_data.check_ray_bitmask |= 1 << index;
                    }
                }
            }
        }

        let opp_king_bits = opp_bits & self.board.get_piece_bitmask(Piece::King);
        let opp_king_index = bits_to_indices(opp_king_bits)[0];
        let opp_king_attack_bitmask = KING_ATTACK_BITMASKS[opp_king_index as usize];

        attack_data.opponent_attack_bitmask = attack_data.opponent_sliding_attack_bitmask
            | attack_data.opponent_knight_attack_bitmask
            | attack_data.opponent_pawn_attack_bitmask
            | opp_king_attack_bitmask;

        attack_data
    }

    fn check_pins_and_checks(
        &self,
        mb_offsets: [i32; 4],
        is_diagonal: bool,
        friendly_king_index: u32,
        attack_data: &mut AttackData,
    ) {
        let friendly_king_mb_index = BOARD_INDEX_TO_MAILBOX_INDEX[friendly_king_index as usize];
        for mb_offset in mb_offsets {
            let mut is_friendly_piece_along_ray = false;
            let mut ray_mask: u64 = 1;
            let mut target_index = ((friendly_king_mb_index as i32) + mb_offset) as u32;
            loop {
                ray_mask |= 1 << target_index;
                let (color, piece) = self.board.get_square(target_index);
                if piece != Piece::Empty {
                    if color == self.turn {
                        // First friendly piece found in this direction so it might be pinned
                        if !is_friendly_piece_along_ray {
                            is_friendly_piece_along_ray = true;
                        // Second friendly piece found so no pins possible
                        } else {
                            break;
                        }
                    } else if piece == Piece::Queen
                        || is_diagonal && piece == Piece::Bishop
                        || !is_diagonal && piece == Piece::Rook
                    {
                        // Friendly piece blocks check, so this is a pin
                        if is_friendly_piece_along_ray {
                            attack_data.pin_exists_in_position = true;
                            attack_data.pin_ray_bitmask |= ray_mask;
                        // No friendly piece to block, so this is a check
                        } else {
                            attack_data.check_ray_bitmask |= ray_mask;
                            // If already in check, then this is a double check
                            attack_data.in_double_check = attack_data.in_check;
                            attack_data.in_check = true;
                        }
                        break;
                    // Opponents piece cannot attack king, so there's no pin or check
                    } else {
                        break;
                    }
                }
                target_index = ((target_index as i32) + mb_offset) as u32
            }
            // Stop searching when in double check since only the king can move anyway
            if attack_data.in_double_check {
                break;
            }
        }
    }

    fn get_sliding_attack_data(
        &self,
        indices: &Vec<u32>,
        mailbox_offsets: [i32; 4],
        friendly_king_index: u32,
    ) -> u64 {
        let mut opp_target_map: u64 = 0;
        for index in indices {
            let mb_index = BOARD_INDEX_TO_MAILBOX_INDEX[*index as usize];
            for mb_offset in mailbox_offsets {
                let mut target_mb_index = ((mb_index as i32) + mb_offset) as u32;
                loop {
                    let target_index = MAILBOX[target_mb_index as usize];
                    if let Some(target_index) = target_index {
                        opp_target_map |= 1 << target_index;
                        if target_index != friendly_king_index
                            && !self.board.is_index_empty(target_index)
                        {
                            break;
                        }
                    } else {
                        break;
                    }
                    target_mb_index = ((target_mb_index as i32) + mb_offset) as u32;
                }
            }
        }
        opp_target_map
    }

    #[allow(dead_code)]
    pub fn generate_legal_moves_at_depth(&self, depth: u32) -> Vec<GameState> {
        let mut game_states = self.generate_legal_moves();

        let mut curr_depth = 1;
        while curr_depth < depth {
            curr_depth += 1;
            game_states = game_states.iter_mut().fold(vec![], |mut next, game_state| {
                let mut next_move_list = game_state.generate_legal_moves();
                next.append(&mut next_move_list);
                next
            });
        }

        game_states
    }

    pub fn generate_pseudo_legal_moves(&self) -> Vec<Move> {
        self.generate_pseudo_legal_moves_inner(self.turn, false)
    }

    fn generate_pseudo_legal_moves_inner(&self, color: Color, attack_only: bool) -> Vec<Move> {
        let mut moves = vec![];

        let board = &self.board;
        for index in 0..64 {
            let (square_color, square_piece) = board.get_square(index);
            if square_color == color {
                moves = match square_piece {
                    Piece::Pawn => self.gen_moves_pawn(color, index, attack_only, moves),
                    Piece::Bishop | Piece::Rook | Piece::Queen => {
                        self.gen_moves_piece(color, &square_piece, index, true, attack_only, moves)
                    }
                    Piece::King | Piece::Knight => {
                        self.gen_moves_piece(color, &square_piece, index, false, attack_only, moves)
                    }
                    Piece::Empty => moves,
                };
                if !attack_only && square_piece == Piece::King {
                    moves = self.gen_castle_moves(color, moves);
                }
            }
        }

        moves
    }

    fn gen_moves_pawn(
        &self,
        color: Color,
        index: u32,
        attack_only: bool,
        mut moves: Vec<Move>,
    ) -> Vec<Move> {
        let one_square_move_index = get_pawn_move_index(&color, index);
        if !attack_only && self.board.is_index_empty(one_square_move_index) {
            if color == Color::White && one_square_move_index < 8
                || color == Color::Black && one_square_move_index > 55
            {
                [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]
                    .iter()
                    .for_each(|piece| {
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
            if let Some(attack_index) = MAILBOX[mailbox_attack_index as usize] {
                if attack_index == en_passant_index {
                    moves.push(Move::en_passant(index, attack_index));
                } else {
                    let is_target_empty = &self.board.is_index_empty(attack_index);
                    if !is_target_empty && !self.board.is_index_of_color(attack_index, color) {
                        if color == Color::White && attack_index < 8
                            || color == Color::Black && attack_index > 55
                        {
                            [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]
                                .iter()
                                .for_each(|piece| {
                                    moves.push(Move::promotion_capture(
                                        index,
                                        attack_index,
                                        *piece,
                                    ));
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

    fn gen_moves_piece(
        &self,
        color: Color,
        piece: &Piece,
        index: u32,
        slide: bool,
        attack_only: bool,
        mut moves: Vec<Move>,
    ) -> Vec<Move> {
        let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index as usize];
        let mailbox_offsets = get_piece_mailbox_direction_offsets(piece);
        for mailbox_offset in mailbox_offsets {
            if slide {
                moves = self.gen_moves_slide_direction(
                    color,
                    index,
                    mailbox_index,
                    *mailbox_offset,
                    attack_only,
                    moves,
                );
            } else {
                moves = self.gen_moves_hop_direction(
                    color,
                    index,
                    mailbox_index,
                    *mailbox_offset,
                    attack_only,
                    moves,
                );
            }
        }
        moves
    }

    fn gen_moves_hop_direction(
        &self,
        color: Color,
        index: u32,
        mailbox_index: u32,
        mailbox_offset: u32,
        attack_only: bool,
        mut moves: Vec<Move>,
    ) -> Vec<Move> {
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
                }
            };
        }
        moves
    }

    fn gen_moves_slide_direction(
        &self,
        color: Color,
        index: u32,
        mailbox_index: u32,
        mailbox_offset: u32,
        attack_only: bool,
        mut moves: Vec<Move>,
    ) -> Vec<Move> {
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

    fn gen_move_from_mailbox(
        &self,
        color: Color,
        target_square_mailbox_index: u32,
        start_index: u32,
    ) -> Option<Move> {
        let target_square_index = MAILBOX[target_square_mailbox_index as usize];
        match target_square_index {
            // Off the board
            None => None,
            Some(target_square_index) => {
                self.gen_move_to_index(color, start_index, target_square_index)
            }
        }
    }

    fn gen_move_to_index(&self, color: Color, from: u32, to: u32) -> Option<Move> {
        if self.board.is_index_empty(to) {
            Some(Move::new(from, to))
        } else if !self.board.is_index_of_color(to, color) {
            Some(Move::capture(from, to))
        } else {
            None
        }
    }
}

fn get_pawn_move_index(turn: &Color, index: u32) -> u32 {
    if turn == &Color::White {
        index - 8
    } else {
        index + 8
    }
}

fn get_pawn_mailbox_attack_indices(turn: &Color, index: u32) -> [u32; 2] {
    let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index as usize];
    if turn == &Color::White {
        [mailbox_index - 9, mailbox_index - 11]
    } else {
        [mailbox_index + 9, mailbox_index + 11]
    }
}

fn get_piece_mailbox_direction_offsets(piece: &Piece) -> &[u32] {
    match piece {
        Piece::Bishop => &DIAGONAL_MAILBOX_DIRECTION_OFFSETS,
        Piece::Rook => &CARDINAL_MAILBOX_DIRECTION_OFFSETS,
        Piece::Queen | Piece::King => &ALL_MAILBOX_DIRECTION_OFFSETS,
        Piece::Knight => &KNIGHT_MAILBOX_DIRECTION_OFFSETS,
        // Pawn moves are calculated differently
        _ => &[],
    }
}

fn bits_to_indices(mut bits: u64) -> Vec<u32> {
    let mut indices: Vec<u32> = vec![];
    while bits != 0 {
        let index = bits.trailing_zeros();
        indices.push(index);
        bits ^= 1 << index;
    }
    indices
}

#[allow(dead_code)]
fn indices_to_bits(indices: Vec<u32>) -> u64 {
    let mut bits = 0;
    for index in indices {
        bits |= 1 << index;
    }
    bits
}

#[cfg(test)]
mod state_tests {
    use super::fen_util::*;

    #[test]
    fn in_check_test() {
        let game_state =
            get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR b KQkq -");
        assert_eq!(game_state.is_in_check(), (true, 4));
        assert!(!game_state.is_opponent_in_check());
    }
}

#[cfg(test)]
mod attack_data_tests {
    use super::fen_util::*;
    use super::*;

    #[test]
    fn king_attack_data() {
        // White's move
        let game_state = get_game_state_from_fen("7k/8/8/8/8/8/6K1/8 w - - 0 1");
        let attack_data = game_state.generate_attack_data();
        assert_eq!(
            AttackData {
                pin_ray_bitmask: 0,
                opponent_sliding_attack_bitmask: 0,
                opponent_knight_attack_bitmask: 0,
                opponent_pawn_attack_bitmask: 0,
                opponent_attack_bitmask: indices_to_bits(vec![6, 14, 15]),
                check_ray_bitmask: 0,
                in_check: false,
                in_double_check: false,
                pin_exists_in_position: false
            },
            attack_data
        );

        // Black's move
        let game_state = get_game_state_from_fen("7k/8/8/8/8/8/6K1/8 b - - 0 1");
        let attack_data = game_state.generate_attack_data();
        assert_eq!(
            AttackData {
                pin_ray_bitmask: 0,
                opponent_sliding_attack_bitmask: 0,
                opponent_knight_attack_bitmask: 0,
                opponent_pawn_attack_bitmask: 0,
                opponent_attack_bitmask: indices_to_bits(vec![45, 46, 47, 53, 55, 61, 62, 63]),
                check_ray_bitmask: 0,
                in_check: false,
                in_double_check: false,
                pin_exists_in_position: false
            },
            attack_data
        );
    }

    #[test]
    fn pawn_attack_data() {
        env_logger::init();
        let game_state = get_game_state_from_fen("7k/8/2p5/4pp2/P3P3/2P5/2P3p1/K7 w - - 0 1");
        let attack_data = game_state.generate_attack_data();
        let king_attack_vec = vec![6, 14, 15];
        let pawn_attack_vec = vec![25, 27, 35, 36, 37, 38, 61, 63];
        let mut all_attack_vec = king_attack_vec;
        all_attack_vec.extend(&pawn_attack_vec);
        assert_eq!(
            attack_data,
            AttackData {
                pin_ray_bitmask: 0,
                opponent_sliding_attack_bitmask: 0,
                opponent_knight_attack_bitmask: 0,
                opponent_pawn_attack_bitmask: indices_to_bits(pawn_attack_vec),
                opponent_attack_bitmask: indices_to_bits(all_attack_vec),
                check_ray_bitmask: 0,
                in_check: false,
                in_double_check: false,
                pin_exists_in_position: false
            }
        );
    }
}

#[cfg(test)]
mod perft_tests {
    // Tests based on https://www.chessprogramming.org/Perft_Results
    use super::fen_util::*;
    use super::*;
    use std::collections::HashMap;

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
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        let moves = game_state.generate_legal_moves();
        assert_eq!(moves.len(), 48);
    }

    #[test]
    fn perft_pos_2_depth_2() {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        let game_states = game_state.generate_legal_moves_at_depth(2);
        assert_eq!(game_states.len(), 2039);
    }

    #[test]
    fn perft_pos_2_depth_3() {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
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
        let game_state = get_game_state_from_fen(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        );
        let moves = game_state.generate_legal_moves();
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn perft_pos_4_depth_2() {
        let game_state = get_game_state_from_fen(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        );
        let game_states = game_state.generate_legal_moves_at_depth(2);
        assert_eq!(game_states.len(), 264);
    }

    #[test]
    fn perft_pos_4_depth_3() {
        let game_state = get_game_state_from_fen(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        );
        let game_states = game_state.generate_legal_moves_at_depth(3);
        assert_eq!(game_states.len(), 9467);
    }

    #[test]
    fn perft_pos_5_depth_1() {
        let game_state =
            get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        let moves = game_state.generate_legal_moves();
        assert_eq!(moves.len(), 44);
    }

    #[test]
    fn perft_pos_5_depth_2() {
        let game_state =
            get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        let game_states = game_state.generate_legal_moves_at_depth(2);
        assert_eq!(game_states.len(), 1486);
    }

    #[test]
    fn perft_pos_5_depth_3() {
        let game_state =
            get_game_state_from_fen("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8");
        let game_states = game_state.generate_legal_moves_at_depth(3);
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
        let game_state = get_game_state_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        );
        let moves = game_state.generate_legal_moves();
        assert_eq!(moves.len(), 46);
    }

    #[test]
    fn perft_pos_6_depth_2() {
        let game_state = get_game_state_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        );
        let game_states = game_state.generate_legal_moves_at_depth(2);
        assert_eq!(game_states.len(), 2079);
    }

    #[test]
    fn perft_pos_6_depth_3() {
        let game_state = get_game_state_from_fen(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        );
        let game_states = game_state.generate_legal_moves_at_depth(3);
        assert_eq!(game_states.len(), 89890);
    }

    #[allow(dead_code)]
    fn print_move_map(game_states: &[GameState]) {
        let mut move_map: HashMap<String, u32> = HashMap::new();
        game_states.iter().for_each(|game_state| {
            let first_move = game_state.move_list.get(0);
            if let Some(first_move) = first_move {
                let from_square = get_square_from_index(first_move.from);
                let to_square = get_square_from_index(first_move.to);
                let key = from_square + &to_square;
                move_map.entry(key).and_modify(|e| *e += 1).or_insert(1);
            }
        });
        println!("{:?}", move_map);
    }
}

#[cfg(test)]
mod benchmark_tests {
    extern crate test;

    use super::fen_util::*;
    use test::Bencher;

    #[bench]
    fn perft_pos_2_pseudo_bench(b: &mut Bencher) {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        b.iter(|| game_state.generate_pseudo_legal_moves());
    }

    #[bench]
    fn perft_pos_2_legal_bench(b: &mut Bencher) {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        b.iter(|| game_state.generate_legal_moves());
    }
}
