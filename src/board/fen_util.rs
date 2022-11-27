use super::*;

pub fn get_game_state_from_fen(fen: &str) -> GameState {
    let mut board = Board::default();
    let mut castle = CastleAvailability {
        white_kingside: false,
        white_queenside: false,
        black_kingside: false,
        black_queenside: false,
    };
    let mut index: usize = 0;
    let mut chars = fen.chars();
    let mut found_slash = true;
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
                board.clear_square(i);
            }
            index = end_index;
        } else {
            if c == ' ' {
                if index != 64 {
                    panic!("Invalid FEN: '{}', board ended too early", fen);
                }
                break;
            } else if c == '/' {
                if index % 8 != 0 {
                    panic!(
                        "Invalid FEN: '{}', rank was too short at index {}",
                        fen, index
                    );
                }
                found_slash = true;
                continue;
            } else if index % 8 == 0 && !found_slash {
                panic!(
                    "Invalid FEN: '{}', rank was too long at index {}",
                    fen, index
                );
            }
            found_slash = false;
            let color = if c.is_uppercase() {
                Color::White
            } else {
                Color::Black
            };
            let piece = match c.to_ascii_lowercase() {
                'p' => Piece::Pawn,
                'b' => Piece::Bishop,
                'n' => Piece::Knight,
                'r' => Piece::Rook,
                'q' => Piece::Queen,
                'k' => Piece::King,
                _ => panic!("Invalid FEN: '{}', invalid character '{}' ", fen, c),
            };
            board.update_square(index, color, piece);
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
    let c = chars.next();
    if c == None {
        return GameState {
            board,
            turn,
            castle,
            ..Default::default()
        };
    }
    let c = c.unwrap();
    if c != ' ' {
        panic!("Invalid FEN: '{}', invalid character '{}' ", fen, c);
    }

    let mut c = chars.next();
    if Some('-') != c {
        while c != None {
            let castle_char = c.unwrap();

            match castle_char {
                'K' => castle.white_kingside = true,
                'Q' => castle.white_queenside = true,
                'k' => castle.black_kingside = true,
                'q' => castle.black_queenside = true,
                ' ' => break,
                _ => panic!(
                    "Invalid FEN: '{}', invalid character '{}' ",
                    fen, castle_char
                ),
            };

            c = chars.next();
        }
    }

    GameState {
        board,
        turn,
        castle,
        ..Default::default()
    }
}

#[allow(dead_code)]
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
        _ => unreachable!(),
    };
    let rank = 8 - index / 8;
    file.to_string() + &rank.to_string()
}

#[allow(dead_code)]
fn game_state_to_fen_string(game_state: &GameState) -> String {
    let board = board_to_fen_string(&game_state.board);
    let turn = match game_state.turn {
        Color::Black => 'b',
        _ => 'w',
    };
    let castle = castle_availability_to_fen(&game_state.castle);
    format!("{} {} {}", board, turn, castle)
}

#[allow(dead_code)]
fn castle_availability_to_fen(castle_availability: &CastleAvailability) -> String {
    let mut output = String::new();
    if castle_availability.white_kingside {
        output.push('K');
    }
    if castle_availability.white_queenside {
        output.push('Q');
    }
    if castle_availability.black_kingside {
        output.push('k');
    }
    if castle_availability.black_queenside {
        output.push('q');
    }

    if output.is_empty() {
        return '-'.to_string();
    }

    output
}

#[allow(dead_code)]
fn board_to_fen_string(board: &Board) -> String {
    let mut board_str = String::new();
    let mut space_count = 0;
    for index in 0..64 {
        let letter = get_fen_char_from_square(board, index);
        if letter == ' ' {
            space_count += 1;
        } else {
            if space_count > 0 {
                board_str.push_str(&space_count.to_string());
                space_count = 0;
            }
            board_str.push(letter);
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

fn get_fen_char_from_square(board: &Board, index: usize) -> char {
    let (color, piece) = board.get_square(index);
    if color == Color::Empty || piece == Piece::Empty {
        return ' ';
    }
    match color {
        Color::Black => match piece {
            Piece::Pawn => 'p',
            Piece::Bishop => 'b',
            Piece::Knight => 'n',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
            Piece::Empty => ' ',
        },
        Color::White => match piece {
            Piece::Pawn => 'P',
            Piece::Bishop => 'B',
            Piece::Knight => 'N',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
            Piece::Empty => ' ',
        },
        Color::Empty => ' ',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_to_fen() {
        let game_state = get_game_state_from_fen(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -",
        );
        let fen = board_to_fen_string(&game_state.board);

        assert_eq!(
            fen,
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R"
        );
    }
}
