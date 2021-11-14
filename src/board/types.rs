use serde::{Serialize, Deserialize};

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
  empties: u64,
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
      empties: 0x00_00_ff_ff_ff_ff_00_00,
    }
  }
}

impl Board {
  pub fn is_index_empty(&self, index: usize) -> bool {
    let bit_mask: u64 = 1 << index;
    self.empties & bit_mask != 0
  }

  pub fn get_square(&self, index: usize) -> (Color, Piece) {
    let bit_mask: u64 = 1 << index;
    let color = if self.black & bit_mask != 0 {
      Color::Black
    } else if self.white & bit_mask != 0 {
      Color::White
    } else {
      Color::Empty
    };
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
      Piece::Empty
    };
    (color, piece)
  }

  pub fn is_index_of_color_and_piece(&self, index: usize, color: Color, piece: Piece) -> bool {
    let bit_mask: u64 = 1 << index;
    let is_color = self.is_index_of_color(index, color);
    if !is_color {
      return false;
    }
    match piece {
      Piece::Pawn => self.pawns & bit_mask != 0,
      Piece::Bishop => self.bishops & bit_mask != 0,
      Piece::Knight => self.knights & bit_mask != 0,
      Piece::Rook => self.rooks & bit_mask != 0,
      Piece::Queen => self.queens & bit_mask != 0,
      Piece::King => self.kings & bit_mask != 0,
      Piece::Empty => self.empties & bit_mask != 0,
    }
  }

  pub fn is_index_of_color(&self, index: usize, color: Color) -> bool {
    let bit_mask: u64 = 1 << index;
    match color {
      Color::Black => self.black & bit_mask != 0,
      Color::White => self.white & bit_mask != 0,
      Color::Empty => self.empties & bit_mask != 0,
    }
  }

  pub fn clear_square(&mut self, index: usize) {
    let bit_mask: u64 = 1 << index;
    let bit_mask_complement: u64 = !bit_mask;
    self.empties |= bit_mask;
    self.white &= bit_mask_complement;
    self.black &= bit_mask_complement;
    self.pawns &= bit_mask_complement;
    self.knights &= bit_mask_complement;
    self.bishops &= bit_mask_complement;
    self.rooks &= bit_mask_complement;
    self.queens &= bit_mask_complement;
    self.kings &= bit_mask_complement;
  }

  pub fn update_square(&mut self, index: usize, color: Color, piece: Piece) {
    let bit_mask: u64 = 1 << index;
    let bit_mask_complement: u64 = !bit_mask;
    match color {
      Color::White => {
        self.white |= bit_mask;
        self.black &= bit_mask_complement;
        self.empties &= bit_mask_complement;
      },
      Color::Black => {
        self.black |= bit_mask;
        self.white &= bit_mask_complement;
        self.empties &= bit_mask_complement;
      },
      Color::Empty => {
        self.empties |= bit_mask;
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
  }

  pub fn castle_black_queenside_open(&self) -> bool {
    let castle_square_bitmask = 0x00_00_00_00_00_00_00_0e;
    !self.empties & castle_square_bitmask == 0
  }

  pub fn castle_black_kingside_open(&self) -> bool {
    let castle_square_bitmask = 0x00_00_00_00_00_00_00_60;
    !self.empties & castle_square_bitmask == 0
  }

  pub fn castle_white_queenside_open(&self) -> bool {
    let castle_square_bitmask = 0x0e_00_00_00_00_00_00_00;
    !self.empties & castle_square_bitmask == 0
  }

  pub fn castle_white_kingside_open(&self) -> bool {
    let castle_square_bitmask = 0x60_00_00_00_00_00_00_00;
    !self.empties & castle_square_bitmask == 0
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GameState {
  pub board: Board,
  pub turn: Color,
  pub move_list: Vec<Move>,
  pub castle: CastleAvailability,
  pub en_passant_index: Option<usize>,
}

impl Default for GameState {
  fn default() -> Self {
    GameState {
      board: Default::default(),
      turn: Color::White,
      move_list: vec!(),
      castle: Default::default(),
      en_passant_index: None,
    }
  }
}
