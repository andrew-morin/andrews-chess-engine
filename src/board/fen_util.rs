use super::constants::*;
use super::types::*;

pub fn get_game_state_from_fen(fen: &str) -> GameState {
  let mut board: Board = INITIAL_BOARD;
  let mut index: usize = 0;
  let mut chars = fen.chars();
  loop {
    let c = chars.next();
    if c == None {
      panic!("Invalid FEN: '{}', ended too early", fen);
    }
    let c = c.unwrap();
    let digit = c.to_digit(10);
    if let Some(digit) = digit {
      let end_index = index + digit as usize;
      for i in index..end_index {
        board[i] = EMPTY_SQUARE;
      }
      index = end_index;
    } else {
      if c == ' ' {
        break;
      } else if c == '/' {
        continue;
      }
      board[index] = match c {
        'p' => BLACK_PAWN,
        'b' => BLACK_BISHOP,
        'n' => BLACK_KNIGHT,
        'r' => BLACK_ROOK,
        'q' => BLACK_QUEEN,
        'k' => BLACK_KING,
        'P' => WHITE_PAWN,
        'B' => WHITE_BISHOP,
        'N' => WHITE_KNIGHT,
        'R' => WHITE_ROOK,
        'Q' => WHITE_QUEEN,
        'K' => WHITE_KING,
        _ => panic!("Invalid FEN: '{}', invalid character '{}' ", fen, c),
      };
      index += 1
    }
  }
  let active_color = chars.next();
  let turn = match active_color {
    Some('w') => Color::White,
    Some('b') => Color::Black,
    Some(c) => panic!("Invalid FEN: '{}', invalid active color '{}'", fen, c),
    None => panic!("Invalid FEN: '{}', ended too early", fen),
  };

  GameState { board, turn, move_list: vec!() }
}

pub fn get_square_from_index(index: usize) -> String {
  let file = match index % 8 {
    0 => 'a',
    1 => 'b',
    2 => 'c',
    3 => 'd',
    4 => 'e',
    5 => 'f',
    6 => 'g',
    7 => 'h',
    _ => unreachable!()
  };
  let rank = 8 - index / 8;
  file.to_string() + &rank.to_string()
}

pub fn board_to_fen_string(board: Board) -> String {
  let mut board_str = String::new();
  let mut space_count = 0;
  for (index, square) in board.iter().enumerate() {
    let letter = get_fen_char_from_square(square);
    if letter == ' ' {
      space_count += 1;
    } else {
      if space_count > 0 {
        board_str.push_str(&space_count.to_string());
        space_count = 0;
      }
      board_str.push(get_fen_char_from_square(square));
    }

    // last square in the rank, but not last rank
    if index % 8 == 7 && index < 63 {
      if space_count > 0 {
        board_str.push_str(&space_count.to_string());
        space_count = 0;
      }
      board_str.push_str("/");
    }
  }

  board_str
}

fn get_fen_char_from_square(square: &Square) -> char {
  if square.empty == true {
    return ' ';
  }
  match square.color {
    Color::Black => match square.piece {
      Piece::Pawn   => 'p',
      Piece::Bishop => 'b',
      Piece::Knight => 'n',
      Piece::Rook   => 'r',
      Piece::Queen  => 'q',
      Piece::King   => 'k',
      Piece::Empty  => ' ',
    },
    Color::White => match square.piece {
      Piece::Pawn   => 'P',
      Piece::Bishop => 'B',
      Piece::Knight => 'N',
      Piece::Rook   => 'R',
      Piece::Queen  => 'Q',
      Piece::King   => 'K',
      Piece::Empty  => ' ',
    },
    Color::Empty => ' ',
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::board::*;

  #[test]
  fn board_to_fen() {
    let game_state = get_game_state_from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
    let fen = board_to_fen_string(game_state.board);

    assert_eq!(fen, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R");
  }
}