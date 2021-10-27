mod board;

use board::types::{Board, Color, Piece, Square};

const EMPTY_SQUARE: Square = Square { empty: true,  color: Color::Empty, piece: Piece::Empty  };
const WHITE_PAWN:   Square = Square { empty: false, color: Color::White, piece: Piece::Pawn   };
const WHITE_BISHOP: Square = Square { empty: false, color: Color::White, piece: Piece::Bishop };
const WHITE_KNIGHT: Square = Square { empty: false, color: Color::White, piece: Piece::Knight };
const WHITE_ROOK:   Square = Square { empty: false, color: Color::White, piece: Piece::Rook   };
const WHITE_QUEEN:  Square = Square { empty: false, color: Color::White, piece: Piece::Queen  };
const WHITE_KING:   Square = Square { empty: false, color: Color::White, piece: Piece::King   };
const BLACK_PAWN:   Square = Square { empty: false, color: Color::Black, piece: Piece::Pawn   };
const BLACK_BISHOP: Square = Square { empty: false, color: Color::Black, piece: Piece::Bishop };
const BLACK_KNIGHT: Square = Square { empty: false, color: Color::Black, piece: Piece::Knight };
const BLACK_ROOK:   Square = Square { empty: false, color: Color::Black, piece: Piece::Rook   };
const BLACK_QUEEN:  Square = Square { empty: false, color: Color::Black, piece: Piece::Queen  };
const BLACK_KING:   Square = Square { empty: false, color: Color::Black, piece: Piece::King   };

const INITIAL_BOARD: Board = [
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
  print_board(INITIAL_BOARD);
}

fn print_board(board: Board) {
  for (index, square) in board.iter().enumerate() {
    let letter = get_fen_char_from_square(square);
    print!("{}", letter);

    // last square in the rank
    if index % 8 == 7 {
      println!();
      println!();
    } else {
      print!(" | ");
    }
  }
}

fn get_fen_char_from_square(square: &Square) -> String {
  if square.empty == true {
    return " ".to_string();
  }
  let letter = match square {
    Square { piece: Piece::Pawn,   .. } => "p",
    Square { piece: Piece::Bishop, .. } => "b",
    Square { piece: Piece::Knight, .. } => "n",
    Square { piece: Piece::Rook,   .. } => "r",
    Square { piece: Piece::Queen,  .. } => "q",
    Square { piece: Piece::King,   .. } => "k",
    Square { piece: Piece::Empty,  .. } => " ",
  };
  match square.color {
    Color::White => letter.to_uppercase(),
    Color::Black => letter.to_string(),
    Color::Empty => " ".to_string(),
  }
}
