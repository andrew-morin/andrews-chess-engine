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

pub const MAILBOX: [Option<usize>; 120] = [
  None, None,     None,     None,     None,     None,     None,     None,     None,     None,
  None, None,     None,     None,     None,     None,     None,     None,     None,     None,
  None, Some(0),  Some(1),  Some(2),  Some(3),  Some(4),  Some(5),  Some(6),  Some(7),  None,
  None, Some(8),  Some(9),  Some(10), Some(11), Some(12), Some(13), Some(14), Some(15), None,
  None, Some(16), Some(17), Some(18), Some(19), Some(20), Some(21), Some(22), Some(23), None,
  None, Some(24), Some(25), Some(26), Some(27), Some(28), Some(29), Some(30), Some(31), None,
  None, Some(32), Some(33), Some(34), Some(35), Some(36), Some(37), Some(38), Some(39), None,
  None, Some(40), Some(41), Some(42), Some(43), Some(44), Some(45), Some(46), Some(47), None,
  None, Some(48), Some(49), Some(50), Some(51), Some(52), Some(53), Some(54), Some(55), None,
  None, Some(56), Some(57), Some(58), Some(59), Some(60), Some(61), Some(62), Some(63), None,
  None, None,     None,     None,     None,     None,     None,     None,     None,     None,
  None, None,     None,     None,     None,     None,     None,     None,     None,     None
];

pub const BOARD_INDEX_TO_MAILBOX_INDEX: [i8; 64] = [
	21, 22, 23, 24, 25, 26, 27, 28,
	31, 32, 33, 34, 35, 36, 37, 38,
	41, 42, 43, 44, 45, 46, 47, 48,
	51, 52, 53, 54, 55, 56, 57, 58,
	61, 62, 63, 64, 65, 66, 67, 68,
	71, 72, 73, 74, 75, 76, 77, 78,
	81, 82, 83, 84, 85, 86, 87, 88,
	91, 92, 93, 94, 95, 96, 97, 98
];
