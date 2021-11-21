use serde::{Serialize, Deserialize};
use super::constants::*;

big_array! { BigArray; }

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Color {
  Empty,
  White,
  Black,
}

impl Color {
  pub fn opposite(&self) -> Color {
    match self {
      Color::White => Color::Black,
      Color::Black => Color::White,
      Color::Empty => Color::Empty,
    }
  }
}

impl Default for Color {
  fn default() -> Self { Color::Empty }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Piece {
  Empty,
  Pawn,
  Bishop,
  Knight,
  Rook,
  Queen,
  King,
}

impl Default for Piece {
  fn default() -> Self { Piece::Empty }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Board {
  black: u64,
  white: u64,
  pawns: u64,
  knights: u64,
  bishops: u64,
  rooks: u64,
  kings: u64,
  queens: u64,
  empty: u64,
}

impl Default for Board {
  fn default() -> Self {
    Board {
      black:   0x00_00_00_00_00_00_ff_ff,
      white:   0xff_ff_00_00_00_00_00_00,
      pawns:   0x00_ff_00_00_00_00_ff_00,
      knights: 0x42_00_00_00_00_00_00_42,
      bishops: 0x24_00_00_00_00_00_00_24,
      rooks:   0x81_00_00_00_00_00_00_81,
      queens:  0x10_00_00_00_00_00_00_10,
      kings:   0x08_00_00_00_00_00_00_08,
      empty: 0x00_00_ff_ff_ff_ff_00_00,
    }
  }
}

impl Board {
  fn get_color_bitmask(&self, color: Color) -> u64 {
    match color {
      Color::Black => self.black,
      Color::White => self.white,
      Color::Empty => self.empty,
    }
  }

  pub fn is_index_empty(&self, index: usize) -> bool {
    let bit_mask: u64 = 1 << index;
    self.empty & bit_mask != 0
  }

  pub fn find_king(&self, color: Color) -> Option<u32> {
    if color == Color::White {
      let king_bit_mask = self.kings & self.white;
      return king_bit_mask.checked_log2();
    } else {
      let king_bit_mask = self.kings & self.black;
      return king_bit_mask.checked_log2();
    }
  }

  pub fn is_pawn_attacking_index(&self, index: usize) -> bool {
    let bit_mask: u64 = 1 << index;
    let color = self.get_square_color_mask(bit_mask);
    let piece_mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[index];
    let opponent_pawn_mailbox_indices = if color == Color::White {
      [piece_mailbox_index - 11, piece_mailbox_index - 9]
    } else {
      [piece_mailbox_index + 9, piece_mailbox_index + 11]
    };
    let opponent_color_bitmask = self.get_color_bitmask(color.opposite());
    let opponent_pawns_bitmask = self.pawns & opponent_color_bitmask;
    opponent_pawn_mailbox_indices.iter().any(|&pawn_mailbox_index| {
      let pawn_index = MAILBOX[pawn_mailbox_index];
      if let Some(pawn_index) = pawn_index {
        opponent_pawns_bitmask & (1 << pawn_index) != 0
      } else {
        false
      }
    })
  }

  pub fn is_knight_attacking_king(&self, color: Color) -> bool {
    let king_index = self.find_king(color);
    if let Some(king_index) = king_index {
      let king_index = king_index as usize;
      let king_mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[king_index];
      let opponent_color_bitmask = self.get_color_bitmask(color.opposite());
      let opponent_knights_bitmask = self.knights & opponent_color_bitmask;
      KNIGHT_MAILBOX_DIRECTION_OFFSETS.iter().any(|mailbox_offset| {
        let target_mailbox_index_plus = king_mailbox_index + mailbox_offset;
        let target_mailbox_index_minus = king_mailbox_index - mailbox_offset;
        [target_mailbox_index_plus, target_mailbox_index_minus].iter().any(|&target_mailbox_index| {
          let target_square_index = MAILBOX[target_mailbox_index];
          if let Some(target_index) = target_square_index {
            opponent_knights_bitmask & (1 << target_index) != 0
          } else {
            false
          }
        })
      })
    } else {
      false
    }
  }

  pub fn is_king_attacking_king(&self, color: Color) -> bool {
    let king_index = self.find_king(color);
    if let Some(king_index) = king_index {
      let king_index = king_index as usize;
      let king_mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[king_index];
      let opponent_color_bitmask = self.get_color_bitmask(color.opposite());
      let opponent_king_bitmask = self.kings & opponent_color_bitmask;
      ALL_MAILBOX_DIRECTION_OFFSETS.iter().any(|mailbox_offset| {
        let target_mailbox_index_plus = king_mailbox_index + mailbox_offset;
        let target_mailbox_index_minus = king_mailbox_index - mailbox_offset;
        [target_mailbox_index_plus, target_mailbox_index_minus].iter().any(|&target_mailbox_index| {
          let target_square_index = MAILBOX[target_mailbox_index];
          if let Some(target_index) = target_square_index {
            opponent_king_bitmask & (1 << target_index) != 0
          } else {
            false
          }
        })
      })
    } else {
      false
    }
  }

  pub fn is_cardinal_slide_piece_attack_king(&self, color: Color) -> bool {
    let king_index = self.find_king(color);
    if let Some(king_index) = king_index {
      let king_index = king_index as usize;
      let king_mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[king_index];
      let opponent_color_bitmask = self.get_color_bitmask(color.opposite());
      let opponent_rook_queen_bitmask = (self.rooks | self.queens) & opponent_color_bitmask;
      CARDINAL_MAILBOX_DIRECTION_OFFSETS.iter().any(|mailbox_offset| {
        let target_mailbox_index_plus = king_mailbox_index + mailbox_offset;
        let target_mailbox_index_minus = king_mailbox_index - mailbox_offset;
        for mut target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
          loop {
            let target_square_index = MAILBOX[target_mailbox_index];
            if let Some(target_index) = target_square_index {
              let is_opponent_piece = opponent_rook_queen_bitmask & (1 << target_index) != 0;
              if is_opponent_piece {
                return true;
              } else if self.is_index_empty(target_index) {
                if target_mailbox_index < king_mailbox_index {
                  target_mailbox_index -= mailbox_offset;
                } else {
                  target_mailbox_index += mailbox_offset;
                }
              } else {
                break;
              }
            } else {
              break;
            }
          }
        }
        false
      })
    } else {
      false
    }
  }

  pub fn is_diagonal_slide_piece_attack_king(&self, color: Color) -> bool {
    let king_index = self.find_king(color);
    if let Some(king_index) = king_index {
      let king_index = king_index as usize;
      let king_mailbox_index = BOARD_INDEX_TO_MAILBOX_INDEX[king_index];
      let opponent_color_bitmask = self.get_color_bitmask(color.opposite());
      let opponent_bishop_queen_bitmask = (self.bishops | self.queens) & opponent_color_bitmask;
      DIAGONAL_MAILBOX_DIRECTION_OFFSETS.iter().any(|mailbox_offset| {
        let target_mailbox_index_plus = king_mailbox_index + mailbox_offset;
        let target_mailbox_index_minus = king_mailbox_index - mailbox_offset;
        for mut target_mailbox_index in [target_mailbox_index_plus, target_mailbox_index_minus] {
          loop {
            let target_square_index = MAILBOX[target_mailbox_index];
            if let Some(target_index) = target_square_index {
              let is_opponent_piece = opponent_bishop_queen_bitmask & (1 << target_index) != 0;
              if is_opponent_piece {
                return true;
              } else if self.is_index_empty(target_index) {
                if target_mailbox_index < king_mailbox_index {
                  target_mailbox_index -= mailbox_offset;
                } else {
                  target_mailbox_index += mailbox_offset;
                }
              } else {
                break;
              }
            } else {
              break;
            }
          }
        }
        false
      })
    } else {
      false
    }
  }

  fn get_square_color_mask(&self, bit_mask: u64) -> Color {
    if self.black & bit_mask != 0 {
      Color::Black
    } else if self.white & bit_mask != 0 {
      Color::White
    } else {
      Color::Empty
    }
  }

  pub fn get_square(&self, index: usize) -> (Color, Piece) {
    let bit_mask: u64 = 1 << index;
    let color = self.get_square_color_mask(bit_mask);
    let piece = if self.pawns & bit_mask != 0 {
      Piece::Pawn
    } else if self.knights & bit_mask != 0 {
      Piece::Knight
    } else if self.bishops & bit_mask != 0 {
      Piece::Bishop
    } else if self.rooks & bit_mask != 0 {
      Piece::Rook
    } else if self.queens & bit_mask != 0 {
      Piece::Queen
    } else if self.kings & bit_mask != 0 {
      Piece::King
    } else {
      debug_assert!(self.empty & bit_mask != 0, "index {} did not appear in any of the piece or empty masks!", index);
      Piece::Empty
    };
    (color, piece)
  }

  pub fn is_index_of_color(&self, index: usize, color: Color) -> bool {
    let bit_mask: u64 = 1 << index;
    self.get_color_bitmask(color) & bit_mask != 0
  }

  pub fn clear_square(&mut self, index: usize) {
    let bit_mask: u64 = 1 << index;
    let bit_mask_complement: u64 = !bit_mask;
    self.empty |= bit_mask;
    self.white &= bit_mask_complement;
    self.black &= bit_mask_complement;
    self.pawns &= bit_mask_complement;
    self.knights &= bit_mask_complement;
    self.bishops &= bit_mask_complement;
    self.rooks &= bit_mask_complement;
    self.queens &= bit_mask_complement;
    self.kings &= bit_mask_complement;
  }

  pub fn move_from_to(&mut self, from: usize, to: usize) {
    let from_bit_mask: u64 = 1 << from;
    let to_bit_mask: u64 = 1 << to;
    let both_bit_mask: u64 = from_bit_mask | to_bit_mask;

    let (from_color, from_piece) = self.get_square(from);
    let (to_color, to_piece) = self.get_square(to);
    let capture = to_color != Color::Empty;
    if from_color == Color::Black {
      self.black ^= both_bit_mask;
      if capture {
        self.white ^= to_bit_mask;
        self.empty |= from_bit_mask;
      } else {
        self.empty ^= both_bit_mask;
      }
    } else {
      self.white ^= both_bit_mask;
      if capture {
        self.black ^= to_bit_mask;
        self.empty |= from_bit_mask;
      } else {
        self.empty ^= both_bit_mask;
      }
    }

    if capture {
      let to_piece_bits = self.get_bits_for_piece(to_piece);
      *to_piece_bits ^= to_bit_mask;
    }
    let from_piece_bits = self.get_bits_for_piece(from_piece);
    *from_piece_bits ^= both_bit_mask;

    self.assert_board_state(format!("from: {}, to: {}", from, to));
  }

  fn get_bits_for_piece(&mut self, piece: Piece) -> &mut u64 {
    match piece {
      Piece::Pawn => &mut self.pawns,
      Piece::Bishop => &mut self.bishops,
      Piece::Knight => &mut self.knights,
      Piece::Rook => &mut self.rooks,
      Piece::Queen => &mut self.queens,
      Piece::King => &mut self.kings,
      Piece::Empty => &mut self.empty,
    }
  }

  pub fn update_square(&mut self, index: usize, color: Color, piece: Piece) {
    let bit_mask: u64 = 1 << index;
    let bit_mask_complement: u64 = !bit_mask;
    match color {
      Color::White => {
        self.white |= bit_mask;
        self.black &= bit_mask_complement;
        self.empty &= bit_mask_complement;
      },
      Color::Black => {
        self.black |= bit_mask;
        self.white &= bit_mask_complement;
        self.empty &= bit_mask_complement;
      },
      Color::Empty => {
        self.empty |= bit_mask;
        self.white &= bit_mask_complement;
        self.black &= bit_mask_complement;
      },
    }

    match piece {
      Piece::Pawn => {
        self.pawns |= bit_mask;
        self.knights &= bit_mask_complement;
        self.bishops &= bit_mask_complement;
        self.rooks &= bit_mask_complement;
        self.queens &= bit_mask_complement;
        self.kings &= bit_mask_complement;
      },
      Piece::Knight => {
        self.pawns &= bit_mask_complement;
        self.knights |= bit_mask;
        self.bishops &= bit_mask_complement;
        self.rooks &= bit_mask_complement;
        self.queens &= bit_mask_complement;
        self.kings &= bit_mask_complement;
      },
      Piece::Bishop => {
        self.pawns &= bit_mask_complement;
        self.knights &= bit_mask_complement;
        self.bishops |= bit_mask;
        self.rooks &= bit_mask_complement;
        self.queens &= bit_mask_complement;
        self.kings &= bit_mask_complement;
      },
      Piece::Rook => {
        self.pawns &= bit_mask_complement;
        self.knights &= bit_mask_complement;
        self.bishops &= bit_mask_complement;
        self.rooks |= bit_mask;
        self.queens &= bit_mask_complement;
        self.kings &= bit_mask_complement;
      },
      Piece::Queen => {
        self.pawns &= bit_mask_complement;
        self.knights &= bit_mask_complement;
        self.bishops &= bit_mask_complement;
        self.rooks &= bit_mask_complement;
        self.queens |= bit_mask;
        self.kings &= bit_mask_complement;
      },
      Piece::King => {
        self.pawns &= bit_mask_complement;
        self.knights &= bit_mask_complement;
        self.bishops &= bit_mask_complement;
        self.rooks &= bit_mask_complement;
        self.queens &= bit_mask_complement;
        self.kings |= bit_mask;
      },
      Piece::Empty => {
        self.pawns &= bit_mask_complement;
        self.knights &= bit_mask_complement;
        self.bishops &= bit_mask_complement;
        self.rooks &= bit_mask_complement;
        self.queens &= bit_mask_complement;
        self.kings &= bit_mask_complement;
      },
    }

    self.assert_board_state(format!("update at index: {}", index));
  }

  pub fn castle_black_queenside_open(&self) -> bool {
    let castle_square_bitmask = 0x00_00_00_00_00_00_00_0e;
    !self.empty & castle_square_bitmask == 0
  }

  pub fn castle_black_kingside_open(&self) -> bool {
    let castle_square_bitmask = 0x00_00_00_00_00_00_00_60;
    !self.empty & castle_square_bitmask == 0
  }

  pub fn castle_white_queenside_open(&self) -> bool {
    let castle_square_bitmask = 0x0e_00_00_00_00_00_00_00;
    !self.empty & castle_square_bitmask == 0
  }

  pub fn castle_white_kingside_open(&self) -> bool {
    let castle_square_bitmask = 0x60_00_00_00_00_00_00_00;
    !self.empty & castle_square_bitmask == 0
  }

  fn assert_board_state(&self, details: String) {
    if cfg!(debug_assertions) {
      debug_assert_eq!(self.white & self.black, u64::MIN, "white and black overlap.\nwhite: {:064b}\nblack: {:064b}\ndetails: {}", self.white, self.black, details);
      debug_assert_eq!(self.white & self.empty, u64::MIN, "white and empty overlap.\nwhite: {:064b}\nempty: {:064b}\ndetails: {}", self.white, self.empty, details);
      debug_assert_eq!(self.black & self.empty, u64::MIN, "black and empty overlap.\nwhite: {:064b}\nempty: {:064b}\ndetails: {}", self.white, self.empty, details);

      debug_assert_eq!(self.pawns ^ self.bishops ^ self.knights ^ self.rooks ^ self.queens ^ self.kings ^ self.empty, u64::MAX, "pieces did not cover all squares or had a duplicate\ndetails: {}", details);
      let piece_bit_masks = [self.pawns, self.bishops, self.knights, self.rooks, self.queens, self.kings, self.empty];
      for i in 0..6 {
        for j in i+1..7 {
          debug_assert_eq!(piece_bit_masks[i] & piece_bit_masks[j], u64::MIN, "two piece bit masks overlap: {} and {}\ndetails: {}", i, j, details);
        }
      }
    }
  }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Move {
  pub from: usize,
  pub to: usize,
  pub capture: bool,
  pub en_passant: bool,
  pub castle: bool,
  pub two_square_pawn_move: bool,
  pub promotion_piece: Option<Piece>,
}

impl Move {
  pub fn new(from: usize, to: usize) -> Move {
    Move { from, to, ..Default::default() }
  }
  pub fn capture(from: usize, to: usize) -> Move {
    Move { from, to, capture: true, ..Default::default() }
  }
  pub fn en_passant(from: usize, to: usize) -> Move {
    Move { from, to, capture: true, en_passant: true, ..Default::default() }
  }
  pub fn castle(from: usize, to: usize) -> Move {
    Move { from, to, castle: true, ..Default::default() }
  }
  pub fn two_square_pawn_move(from: usize, to: usize) -> Move {
    Move { from, to, two_square_pawn_move: true, ..Default::default() }
  }
  pub fn promotion(from: usize, to: usize, promotion_piece: Piece) -> Move {
    Move { from, to, promotion_piece: Some(promotion_piece), ..Default::default() }
  }
  pub fn promotion_capture(from: usize, to: usize, promotion_piece: Piece) -> Move {
    Move { from, to, capture: true, promotion_piece: Some(promotion_piece), ..Default::default() }
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CastleAvailability {
  pub white_kingside: bool,
  pub white_queenside: bool,
  pub black_kingside: bool,
  pub black_queenside: bool,
}

impl Default for CastleAvailability {
  fn default() -> Self {
    CastleAvailability {
      white_kingside: true,
      white_queenside: true,
      black_kingside: true,
      black_queenside: true,
    }
  }
}
