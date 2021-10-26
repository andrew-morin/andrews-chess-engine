use phf::{phf_map, Map};

enum Color {
  EMPTY,
  WHITE,
  BLACK,
}

enum Piece {
  EMPTY,
  PAWN,
  BISHOP,
  KNIGHT,
  ROOK,
  QUEEN,
  KING,
}

struct Square {
  empty: bool,
  color: Color,
  piece: Piece,
}

const EMPTY_SQUARE: Square = Square { empty: true, color: Color::EMPTY, piece: Piece::EMPTY };
const WHITE_PAWN: Square = Square { empty: false, color: Color::WHITE, piece: Piece::PAWN };
const WHITE_BISHOP: Square = Square { empty: false, color: Color::WHITE, piece: Piece::BISHOP };
const WHITE_KNIGHT: Square = Square { empty: false, color: Color::WHITE, piece: Piece::KNIGHT };
const WHITE_ROOK: Square = Square { empty: false, color: Color::WHITE, piece: Piece::ROOK };
const WHITE_QUEEN: Square = Square { empty: false, color: Color::WHITE, piece: Piece::QUEEN };
const WHITE_KING: Square = Square { empty: false, color: Color::WHITE, piece: Piece::KING };
const BLACK_PAWN: Square = Square { empty: false, color: Color::BLACK, piece: Piece::PAWN };
const BLACK_BISHOP: Square = Square { empty: false, color: Color::BLACK, piece: Piece::BISHOP };
const BLACK_KNIGHT: Square = Square { empty: false, color: Color::BLACK, piece: Piece::KNIGHT };
const BLACK_ROOK: Square = Square { empty: false, color: Color::BLACK, piece: Piece::ROOK };
const BLACK_QUEEN: Square = Square { empty: false, color: Color::BLACK, piece: Piece::QUEEN };
const BLACK_KING: Square = Square { empty: false, color: Color::BLACK, piece: Piece::KING };

static INITIAL_FEN_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const INITIAL_BOARD: [Square; 64] = [
  BLACK_ROOK  , BLACK_KNIGHT, BLACK_BISHOP, BLACK_QUEEN , BLACK_KING  , BLACK_BISHOP, BLACK_KNIGHT, BLACK_ROOK  ,
  BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  , BLACK_PAWN  ,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE, EMPTY_SQUARE,
  WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  , WHITE_PAWN  ,
  WHITE_ROOK  , WHITE_KNIGHT, WHITE_BISHOP, WHITE_QUEEN , WHITE_KING  , WHITE_BISHOP, WHITE_KNIGHT, WHITE_ROOK  ,
];

fn main() {
    println!("Hello, world!");
}

fn get_square_from_fen_char(c: char) -> Square {
  match c {
    'P' => WHITE_PAWN,
    'B' => WHITE_BISHOP,
    'N' => WHITE_KNIGHT,
    'R' => WHITE_ROOK,
    'Q' => WHITE_QUEEN,
    'K' => WHITE_KING,
    'p' => BLACK_PAWN,
    'b' => BLACK_BISHOP,
    'n' => BLACK_KNIGHT,
    'r' => BLACK_ROOK,
    'q' => BLACK_QUEEN,
    'k' => BLACK_KING,
    _ => EMPTY_SQUARE,
  }
}
