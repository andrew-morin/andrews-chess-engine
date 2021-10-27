use super::types::{Board, Color, Piece, Square};

pub const EMPTY_SQUARE: Square = Square { empty: true,  color: Color::Empty, piece: Piece::Empty  };
pub const WHITE_PAWN:   Square = Square { empty: false, color: Color::White, piece: Piece::Pawn   };
pub const WHITE_BISHOP: Square = Square { empty: false, color: Color::White, piece: Piece::Bishop };
pub const WHITE_KNIGHT: Square = Square { empty: false, color: Color::White, piece: Piece::Knight };
pub const WHITE_ROOK:   Square = Square { empty: false, color: Color::White, piece: Piece::Rook   };
pub const WHITE_QUEEN:  Square = Square { empty: false, color: Color::White, piece: Piece::Queen  };
pub const WHITE_KING:   Square = Square { empty: false, color: Color::White, piece: Piece::King   };
pub const BLACK_PAWN:   Square = Square { empty: false, color: Color::Black, piece: Piece::Pawn   };
pub const BLACK_BISHOP: Square = Square { empty: false, color: Color::Black, piece: Piece::Bishop };
pub const BLACK_KNIGHT: Square = Square { empty: false, color: Color::Black, piece: Piece::Knight };
pub const BLACK_ROOK:   Square = Square { empty: false, color: Color::Black, piece: Piece::Rook   };
pub const BLACK_QUEEN:  Square = Square { empty: false, color: Color::Black, piece: Piece::Queen  };
pub const BLACK_KING:   Square = Square { empty: false, color: Color::Black, piece: Piece::King   };

pub const INITIAL_BOARD: Board = [
  BLACK_ROOK  , BLACK_KNIGHT, BLACK_BISHOP, BLACK_QUEEN , BLACK_KING  , BLACK_BISHOP, BLACK_KNIGHT, BLACK_ROOK  ,
  BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  ,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  ,
  WHITE_ROOK  , WHITE_KNIGHT, WHITE_BISHOP, WHITE_QUEEN , WHITE_KING  , WHITE_BISHOP, WHITE_KNIGHT, WHITE_ROOK  ,
];