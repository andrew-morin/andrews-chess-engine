import * as wasm from 'wasm-andrews-chess-engine';
import black_pawn from './piece_images/black_pawn.svg';
import black_bishop from './piece_images/black_bishop.svg';
import black_knight from './piece_images/black_knight.svg';
import black_rook from './piece_images/black_rook.svg';
import black_queen from './piece_images/black_queen.svg';
import black_king from './piece_images/black_king.svg';
import white_pawn from './piece_images/white_pawn.svg';
import white_bishop from './piece_images/white_bishop.svg';
import white_knight from './piece_images/white_knight.svg';
import white_rook from './piece_images/white_rook.svg';
import white_queen from './piece_images/white_queen.svg';
import white_king from './piece_images/white_king.svg';

const fen = wasm.print_board();

const mainElement = document.getElementById('main');
const table = document.createElement('table');
table.classList.add('board');
const tbody = document.createElement('tbody');

const fenRows = fen.split('/');
fenRows.forEach(fenRow => {
  const row = document.createElement('tr');

  for (let i = 0; i < fenRow.length; i++) {
    const char = fenRow.charAt(i);
    const int = parseInt(char, 10);
    if (isNaN(int)) {
      const cell = document.createElement('td');
      const img = document.createElement('img');
      img.src = getPieceImage(char);
      cell.appendChild(img);
      row.appendChild(cell);
    } else {
      for (let j = 0; j < int; j++) {
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
  switch(char) {
    case 'p': return black_pawn;
    case 'b': return black_bishop;
    case 'n': return black_knight;
    case 'r': return black_rook;
    case 'q': return black_queen;
    case 'k': return black_king;
    case 'P': return white_pawn;
    case 'B': return white_bishop;
    case 'N': return white_knight;
    case 'R': return white_rook;
    case 'Q': return white_queen;
    case 'K': return white_king;
  }
}
