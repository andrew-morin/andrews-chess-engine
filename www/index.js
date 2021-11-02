import * as wasm from 'wasm-andrews-chess-engine';
import blackPawn from './piece_images/black_pawn.svg';
import blackBishop from './piece_images/black_bishop.svg';
import blackKnight from './piece_images/black_knight.svg';
import blackRook from './piece_images/black_rook.svg';
import blackQueen from './piece_images/black_queen.svg';
import blackKing from './piece_images/black_king.svg';
import whitePawn from './piece_images/white_pawn.svg';
import whiteBishop from './piece_images/white_bishop.svg';
import whiteKnight from './piece_images/white_knight.svg';
import whiteRook from './piece_images/white_rook.svg';
import whiteQueen from './piece_images/white_queen.svg';
import whiteKing from './piece_images/white_king.svg';

const fen = wasm.print_board();

const mainElement = document.getElementById('main');
const table = document.createElement('table');
table.classList.add('board');
const tbody = document.createElement('tbody');

const fenRows = fen.split('/');
fenRows.forEach((fenRow) => {
  const row = document.createElement('tr');

  for (let i = 0; i < fenRow.length; i += 1) {
    const char = fenRow.charAt(i);
    const int = parseInt(char, 10);
    if (Number.isNaN(int)) {
      const cell = getPieceCell(char);
      row.appendChild(cell);
    } else {
      for (let j = 0; j < int; j += 1) {
        const cell = document.createElement('td');
        row.appendChild(cell);
      }
    }
  }

  tbody.appendChild(row);
});

table.appendChild(tbody);
mainElement.appendChild(table);

function getPieceImage(char) {
  switch (char) {
    case 'p': return blackPawn;
    case 'b': return blackBishop;
    case 'n': return blackKnight;
    case 'r': return blackRook;
    case 'q': return blackQueen;
    case 'k': return blackKing;
    case 'P': return whitePawn;
    case 'B': return whiteBishop;
    case 'N': return whiteKnight;
    case 'R': return whiteRook;
    case 'Q': return whiteQueen;
    case 'K': return whiteKing;
    default: return null;
  }
}

function getPieceCell(char) {
  const cell = document.createElement('td');
  const img = document.createElement('img');
  img.src = getPieceImage(char);
  cell.appendChild(img);

  return cell;
}
