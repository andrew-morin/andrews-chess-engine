pub mod constants;
pub mod fen_util;
pub mod types;

use std::collections::HashMap;

use constants::*;
use serde::{Deserialize, Serialize};
use types::*;

const GLOBAL_WITH_INFO: bool = true;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
    pub board: Board,
    pub turn: Color,
    pub castle: CastleAvailability,
    pub en_passant_index: Option<usize>,
    pub halfmove_counter: u8,
}

#[derive(Debug, Default)]
pub struct GameStateInfo {
    // Map of piece index to mask of squares in line of pin
    pub pins: HashMap<usize, u64>,
    pub in_check: bool,
    pub check_mask: u64,
    pub attack_mask: u64,
    pub in_double_check: bool,
}

impl Default for GameState {
    fn default() -> Self {
        GameState {
            board: Default::default(),
            turn: Color::White,
            castle: Default::default(),
            en_passant_index: None,
            halfmove_counter: 0,
        }
    }
}

impl GameState {
    pub fn is_opponent_in_check(&self) -> bool {
        self.is_in_check_inner(self.turn.opposite())
    }

    pub fn is_in_check(&self) -> bool {
        self.is_in_check_inner(self.turn)
    }

    fn is_in_check_inner(&self, color: Color) -> bool {
        let king_index = self.board.find_king(color);
        self.board.is_index_under_attack(king_index)
    }

    pub fn perform_move(&self, next_move: Move) -> GameState {
        let mut game_state_clone = self.clone();
        let Move { from, to, .. } = next_move;

        game_state_clone.board.move_from_to(from, to);
        if next_move.castle {
            // Move the rook
            if to == 2 {
                game_state_clone.board.move_from_to(0, 3);
            } else if to == 6 {
                game_state_clone.board.move_from_to(7, 5);
            } else if to == 58 {
                game_state_clone.board.move_from_to(56, 59);
            } else if to == 62 {
                game_state_clone.board.move_from_to(63, 61);
            }
        }

        game_state_clone.update_castle_availability(from, to);

        if next_move.en_passant {
            let captured_pawn_index = if from > to { to + 8 } else { to - 8 };
            game_state_clone.board.clear_square(captured_pawn_index);
        }

        if next_move.two_square_pawn_move {
            game_state_clone.en_passant_index = Some((from + to) / 2);
        } else {
            game_state_clone.en_passant_index = None;
        }

        // If the move is a capture or a pawn move, reset the halfmove counter. Otherwise, increment it
        if next_move.capture || game_state_clone.board.get_square(from).1 == Piece::Pawn {
            game_state_clone.halfmove_counter = 0;
        } else {
            game_state_clone.halfmove_counter += 1;
        }

        if let Some(promotion_piece) = next_move.promotion_piece {
            game_state_clone
                .board
                .update_square(to, game_state_clone.turn, promotion_piece);
        }

        game_state_clone.turn = game_state_clone.turn.opposite();

        game_state_clone
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

    pub fn generate_game_state_info(&self) -> GameStateInfo {
        let mut game_state_info = GameStateInfo::default();
        self.generate_attack_mask(&mut game_state_info);
        self.generate_pin_masks(&mut game_state_info);
        game_state_info
    }

    pub fn generate_attack_mask(&self, game_state_info: &mut GameStateInfo) {
        let opponent_bitmask = self.board.get_color_bitmask(self.turn.opposite());
        (0..64).for_each(|index| {
            if (1 << index) & opponent_bitmask > 0 {
                let mailbox_start_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
                let (_color, piece) = self.board.get_square(index);
                if piece == Piece::Pawn {
                    let pawn_mailbox_offsets =
                        get_pawn_mailbox_attack_indices(&self.turn.opposite(), index);
                    for target_mailbox_index in pawn_mailbox_offsets {
                        if let Some(target_index) = MAILBOX[target_mailbox_index] {
                            game_state_info.attack_mask |= 1 << target_index;
                        }
                    }
                }
                let mailbox_direction_offsets = get_piece_mailbox_direction_offsets(&piece);
                for mailbox_offset in mailbox_direction_offsets {
                    let target_mailbox_index_plus = mailbox_start_index + mailbox_offset;
                    let target_mailbox_index_minus = mailbox_start_index - mailbox_offset;
                    for mut target_mailbox_index in
                        [target_mailbox_index_plus, target_mailbox_index_minus]
                    {
                        while let Some(target_index) = MAILBOX[target_mailbox_index] {
                            game_state_info.attack_mask |= 1 << target_index;
                            let (attacked_color, attacked_piece) = self.board.get_square(target_index);
                            if !piece.is_slide()
                                || attacked_piece != Piece::Empty
                                    // Ignore king for calculating attack mak
                                    && !(attacked_color == self.turn && attacked_piece == Piece::King)
                            {
                                break;
                            }
                            if target_mailbox_index < mailbox_start_index {
                                target_mailbox_index -= mailbox_offset;
                            } else {
                                target_mailbox_index += mailbox_offset;
                            }
                        }
                    }
                }
            }
        });
    }

    pub fn generate_pin_masks(&self, game_state_info: &mut GameStateInfo) {
        let king_index = self.board.find_king(self.turn);
        let mailbox_start_index = BOARD_INDEX_TO_MAILBOX_INDEX[king_index];
        // Check Bishops, Rooks, and Queens
        for piece_to_check in [Piece::Rook, Piece::Bishop] {
            let mailbox_direction_offsets = get_piece_mailbox_direction_offsets(&piece_to_check);
            for mailbox_offset in mailbox_direction_offsets {
                let target_mailbox_index_plus = mailbox_start_index + mailbox_offset;
                let target_mailbox_index_minus = mailbox_start_index - mailbox_offset;
                for mut target_mailbox_index in
                    [target_mailbox_index_plus, target_mailbox_index_minus]
                {
                    let mut found_piece_index = None;
                    let mut mask: u64 = 0;
                    while let Some(target_index) = MAILBOX[target_mailbox_index] {
                        mask |= 1 << target_index;
                        let (color, piece) = self.board.get_square(target_index);
                        if color == self.turn {
                            if found_piece_index.is_some() {
                                // Found second piece, no pin possible
                                break;
                            } else {
                                found_piece_index = Some(target_index);
                            }
                        } else if color == self.turn.opposite() {
                            if piece == Piece::Queen || piece == piece_to_check {
                                if let Some(found_piece_index) = found_piece_index {
                                    game_state_info.pins.insert(found_piece_index, mask);
                                } else {
                                    game_state_info.in_double_check = game_state_info.in_check;
                                    game_state_info.in_check = true;
                                    if game_state_info.in_double_check {
                                        // Double check means only king moves are valid; no need to check anything more
                                        return;
                                    } else {
                                        game_state_info.check_mask = mask;
                                    }
                                }
                            }
                            break;
                        }
                        if target_mailbox_index < mailbox_start_index {
                            target_mailbox_index -= mailbox_offset;
                        } else {
                            target_mailbox_index += mailbox_offset;
                        }
                    }
                }
            }
        }

        // Check Knights
        for mailbox_offset in KNIGHT_MAILBOX_DIRECTION_OFFSETS {
            let target_mailbox_index_plus = mailbox_start_index + mailbox_offset;
            let target_mailbox_index_minus = mailbox_start_index - mailbox_offset;
            for target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
                if let Some(target_index) = MAILBOX[target_mailbox_index] {
                    let (color, piece) = self.board.get_square(target_index);
                    if color == self.turn.opposite() && piece == Piece::Knight {
                        game_state_info.in_double_check = game_state_info.in_check;
                        game_state_info.in_check = true;
                        if game_state_info.in_double_check {
                            // Double check means only king moves are valid; no need to check anything more
                            return;
                        } else {
                            game_state_info.check_mask = 1 << target_index;
                        }
                    }
                }
            }
        }

        // Check Pawns
        let target_mailbox_indices = get_pawn_mailbox_attack_indices(&self.turn, king_index);
        for target_mailbox_index in target_mailbox_indices {
            if let Some(target_index) = MAILBOX[target_mailbox_index] {
                let (color, piece) = self.board.get_square(target_index);
                if color == self.turn.opposite() && piece == Piece::Pawn {
                    game_state_info.in_check = true;
                    game_state_info.check_mask = 1 << target_index;
                    // Can't double check with pawn, no need to check more
                    break;
                }
            }
        }
    }

    // Generates pseudo legal moves, then removes the ones with the king in check.
    // This is slow and should be updated later.
    pub fn generate_legal_states(&self) -> Vec<GameState> {
        self.generate_legal_states_inner(GLOBAL_WITH_INFO)
    }

    pub fn generate_legal_states_inner(&self, with_info: bool) -> Vec<GameState> {
        let pseudo_legal_moves = self.generate_pseudo_legal_moves(with_info);
        if with_info {
            return pseudo_legal_moves
                .iter()
                .map(|m| self.perform_move(*m))
                .collect();
        }
        let current_is_in_check = self.is_in_check();
        pseudo_legal_moves
            .iter()
            .filter(|&next_move| !next_move.castle || !current_is_in_check)
            .filter_map(|&next_move| {
                let game_state_clone = self.perform_move(next_move);
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
                } else if game_state_clone.is_opponent_in_check() {
                    None
                } else {
                    Some(game_state_clone)
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn generate_legal_moves_at_depth(&self, depth: usize) -> Vec<GameState> {
        self.generate_legal_moves_at_depth_inner(depth, true)
    }

    pub fn generate_legal_moves_at_depth_inner(
        &self,
        depth: usize,
        with_info: bool,
    ) -> Vec<GameState> {
        let mut game_states = self.generate_legal_states_inner(with_info);

        let mut curr_depth = 1;
        while curr_depth < depth {
            curr_depth += 1;
            game_states = game_states.iter_mut().fold(vec![], |mut next, game_state| {
                let mut next_game_state = game_state.generate_legal_states_inner(with_info);
                next.append(&mut next_game_state);
                next
            });
        }

        game_states
    }

    pub fn generate_pseudo_legal_moves(&self, with_info: bool) -> Vec<Move> {
        let game_state_info = if with_info {
            self.generate_game_state_info()
        } else {
            GameStateInfo::default()
        };
        self.generate_pseudo_legal_moves_inner(self.turn, &game_state_info)
    }

    fn generate_pseudo_legal_moves_inner(
        &self,
        color: Color,
        game_state_info: &GameStateInfo,
    ) -> Vec<Move> {
        let board = &self.board;
        // In double check, only calculate king moves
        if game_state_info.in_double_check {
            let king_index = board.find_king(color);
            return self.gen_moves_piece(color, &Piece::King, king_index, vec![], game_state_info);
        }
        let mut moves = vec![];
        let mask_to_check = board.get_color_bitmask(color);
        for index in 0..64 {
            if (1 << index) & mask_to_check > 0 {
                let (_square_color, square_piece) = board.get_square(index);
                moves = match square_piece {
                    Piece::Pawn => self.gen_moves_pawn(color, index, moves, game_state_info),
                    Piece::Empty => moves,
                    _ => self.gen_moves_piece(color, &square_piece, index, moves, game_state_info),
                };
                // Can't castle out of check
                if square_piece == Piece::King && !game_state_info.in_check {
                    moves = self.gen_castle_moves(color, moves, game_state_info);
                }
            }
        }

        moves
    }

    fn gen_moves_pawn(
        &self,
        color: Color,
        index: usize,
        mut moves: Vec<Move>,
        game_state_info: &GameStateInfo,
    ) -> Vec<Move> {
        let one_square_move_index = get_pawn_move_index(&color, index);
        if self.board.is_index_empty(one_square_move_index) {
            let is_one_square_move_legal =
                self.is_move_legal(index, one_square_move_index, &Piece::Pawn, game_state_info);
            if color == Color::White && one_square_move_index < 8
                || color == Color::Black && one_square_move_index > 55
            {
                if is_one_square_move_legal {
                    [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen]
                        .iter()
                        .for_each(|piece| {
                            moves.push(Move::promotion(index, one_square_move_index, *piece));
                        });
                }
            } else {
                if is_one_square_move_legal {
                    moves.push(Move::new(index, one_square_move_index));
                }
                if color == Color::White && index >= 48 || color == Color::Black && index < 16 {
                    let two_square_move_index = get_pawn_move_index(&color, one_square_move_index);
                    if self.board.is_index_empty(two_square_move_index)
                        && self.is_move_legal(
                            index,
                            two_square_move_index,
                            &Piece::Pawn,
                            game_state_info,
                        )
                    {
                        moves.push(Move::two_square_pawn_move(index, two_square_move_index));
                    }
                }
            }
        }

        let mailbox_attack_indices = get_pawn_mailbox_attack_indices(&color, index);
        let en_passant_index = self.en_passant_index.unwrap_or(100);
        for mailbox_attack_index in mailbox_attack_indices {
            if let Some(attack_index) = MAILBOX[mailbox_attack_index] {
                if self.is_move_legal(index, attack_index, &Piece::Pawn, game_state_info) {
                    if attack_index == en_passant_index {
                        // Maybe optimizable, but rare enough that it's not worth it
                        // (at most 2 possible en_passant moves)
                        let game_state_clone =
                            self.perform_move(Move::en_passant(index, attack_index));
                        if !game_state_clone.is_opponent_in_check() {
                            moves.push(Move::en_passant(index, attack_index));
                        }
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
        }

        moves
    }

    fn gen_moves_piece(
        &self,
        color: Color,
        piece: &Piece,
        index: usize,
        mut moves: Vec<Move>,
        game_state_info: &GameStateInfo,
    ) -> Vec<Move> {
        let mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
        let mailbox_offsets = get_piece_mailbox_direction_offsets(piece);
        for mailbox_offset in mailbox_offsets {
            if piece.is_slide() {
                moves = self.gen_moves_slide_direction(
                    color,
                    piece,
                    index,
                    mailbox_index,
                    *mailbox_offset,
                    moves,
                    game_state_info,
                );
            } else {
                moves = self.gen_moves_hop_direction(
                    color,
                    piece,
                    index,
                    mailbox_index,
                    *mailbox_offset,
                    moves,
                    game_state_info,
                );
            }
        }
        moves
    }

    fn gen_moves_hop_direction(
        &self,
        color: Color,
        piece: &Piece,
        index: usize,
        mailbox_index: usize,
        mailbox_offset: usize,
        mut moves: Vec<Move>,
        game_state_info: &GameStateInfo,
    ) -> Vec<Move> {
        let target_mailbox_index_plus = mailbox_index + mailbox_offset;
        let target_mailbox_index_minus = mailbox_index - mailbox_offset;
        for target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
            let generated_move = self.gen_move_from_mailbox(color, target_mailbox_index, index);
            match generated_move {
                // Off the board
                None => (),
                Some(generated_move) => {
                    if self.is_move_legal(index, generated_move.to, piece, game_state_info) {
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
        piece: &Piece,
        index: usize,
        mailbox_index: usize,
        mailbox_offset: usize,
        mut moves: Vec<Move>,
        game_state_info: &GameStateInfo,
    ) -> Vec<Move> {
        let target_mailbox_index_plus = mailbox_index + mailbox_offset;
        let target_mailbox_index_minus = mailbox_index - mailbox_offset;
        for mut target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
            while let Some(generated_move) =
                self.gen_move_from_mailbox(color, target_mailbox_index, index)
            {
                let capture = generated_move.capture;
                if self.is_move_legal(index, generated_move.to, piece, game_state_info) {
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
        moves
    }

    fn gen_castle_moves(
        &self,
        color: Color,
        mut moves: Vec<Move>,
        game_state_info: &GameStateInfo,
    ) -> Vec<Move> {
        if !game_state_info.in_check {
            if color == Color::Black {
                if self.castle.black_queenside
                    && self.board.castle_black_queenside_open(game_state_info)
                {
                    moves.push(Move::castle(4, 2));
                }
                if self.castle.black_kingside
                    && self.board.castle_black_kingside_open(game_state_info)
                {
                    moves.push(Move::castle(4, 6));
                }
            } else {
                if self.castle.white_queenside
                    && self.board.castle_white_queenside_open(game_state_info)
                {
                    moves.push(Move::castle(60, 58));
                }
                if self.castle.white_kingside
                    && self.board.castle_white_kingside_open(game_state_info)
                {
                    moves.push(Move::castle(60, 62));
                }
            }
        }
        moves
    }

    fn is_move_legal(
        &self,
        from_index: usize,
        to_index: usize,
        piece: &Piece,
        game_state_info: &GameStateInfo,
    ) -> bool {
        let to_mask = 1 << to_index;
        if piece == &Piece::King && game_state_info.attack_mask & to_mask > 0 {
            // Moving into check, invalid
            false
        } else if game_state_info.in_check {
            // Already checked above that the king is not moving into check
            return piece == &Piece::King
            // Pinned pieces cannot block check
                || !game_state_info.pins.contains_key(&from_index)
                    && game_state_info.check_mask & to_mask > 0;
        } else if let Some(pin_mask) = game_state_info.pins.get(&from_index) {
            // Pinned knights cannot move
            return piece != &Piece::Knight && pin_mask & to_mask > 0;
        } else {
            true
        }
    }

    fn gen_move_from_mailbox(
        &self,
        color: Color,
        target_square_mailbox_index: usize,
        start_index: usize,
    ) -> Option<Move> {
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
        let game_state =
            get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR b KQkq -");
        assert!(game_state.is_in_check());
        assert!(!game_state.is_opponent_in_check());
    }

    #[test]
    fn pawn_move_reset_halfmove_counter() {
        let game_state =
            get_game_state_from_fen("rnbqkbnr/ppp1pppp/3p4/1B6/8/4P3/PPPP1PPP/RNBQK1NR b KQkq - 5");
        assert_eq!(game_state.halfmove_counter, 5);
    }
}

#[cfg(test)]
mod perft_tests {
    // Tests based on https://www.chessprogramming.org/Perft_Results
    use super::fen_util::*;
    use super::*;

    #[test]
    fn perft_pos_1_depth_1() {
        let moves = GameState::default().generate_legal_states();
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
        let moves = game_state.generate_legal_states();
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
        let moves = game_state.generate_legal_states();
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
        let moves = game_state.generate_legal_states();
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
        let moves = game_state.generate_legal_states();
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
        let moves = game_state.generate_legal_states();
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
}

#[cfg(test)]
mod benchmark_tests {
    extern crate test;

    use super::{fen_util::*, GameState, Move, GLOBAL_WITH_INFO};
    use test::Bencher;

    #[bench]
    fn perform_move(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| game_state.perform_move(Move::new(52, 36)));
    }

    #[bench]
    fn start_pos_pseudo_bench(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| game_state.generate_pseudo_legal_moves(GLOBAL_WITH_INFO));
    }

    #[bench]
    fn start_pos_legal_bench(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| game_state.generate_legal_states());
    }

    #[bench]
    fn perft_pos_2_pseudo_bench(b: &mut Bencher) {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        b.iter(|| game_state.generate_pseudo_legal_moves(GLOBAL_WITH_INFO));
    }

    #[bench]
    fn perft_pos_2_legal_bench(b: &mut Bencher) {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        b.iter(|| game_state.generate_legal_states());
    }

    #[bench]
    fn start_pos_in_check(b: &mut Bencher) {
        let game_state = GameState::default();
        b.iter(|| game_state.is_in_check());
    }
}
