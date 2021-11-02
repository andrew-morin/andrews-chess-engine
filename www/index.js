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

const SQUARE_IMAGE_MAP = {
  White: {
    Pawn: whitePawn,
    Bishop: whiteBishop,
    Knight: whiteKnight,
    Rook: whiteRook,
    Queen: whiteQueen,
    King: whiteKing,
  },
  Black: {
    Pawn: blackPawn,
    Bishop: blackBishop,
    Knight: blackKnight,
    Rook: blackRook,
    Queen: blackQueen,
    King: blackKing,
  },
};

const gameState = wasm.get_initial_game_state();
const pseudoLegalMoves = wasm.get_pseudo_legal_moves(gameState);

const mainElement = document.getElementById('main');
const table = document.createElement('table');
table.classList.add('board');
const tbody = document.createElement('tbody');

let row;
gameState.board.forEach((square, index) => {
  const fileIndex = index % 8;
  if (fileIndex === 0) {
    row = document.createElement('tr');
  }

  const cell = square.empty ? document.createElement('td') : getSquareCell(square);
  cell.dataset.index = index;
  cell.addEventListener('click', getOnClick(index, square));
  row.appendChild(cell);

  tbody.appendChild(row);
});

table.appendChild(tbody);
mainElement.appendChild(table);

let selectedPiece = null;
let validTargetSquares = null;

document.addEventListener('click', () => {
  const oldSelectedPiece = selectedPiece;
  const oldValidTargetSquares = validTargetSquares;
  selectedPiece = null;
  validTargetSquares = null;
  updateValidCells(oldSelectedPiece, oldValidTargetSquares);
});

function getPieceImage(square) {
  return SQUARE_IMAGE_MAP[square.color][square.piece];
}

function getSquareCell(square) {
  const cell = document.createElement('td');
  const img = document.createElement('img');
  img.src = getPieceImage(square);
  cell.appendChild(img);

  return cell;
}

function getOnClick(index, square) {
  return (event) => {
    const oldValidTargetSquares = validTargetSquares;
    const oldSelectedPiece = selectedPiece;
    if (square.empty || square.color !== gameState.turn || index === selectedPiece) {
      selectedPiece = null;
      validTargetSquares = null;
    } else if (selectedPiece != null && validTargetSquares.includes(index)) {
      //  make move
      selectedPiece = null;
      validTargetSquares = null;
    } else {
      updateSelectedPiece(index);
    }
    updateValidCells(oldSelectedPiece, oldValidTargetSquares);
    event.stopPropagation();
  };
}

function updateValidCells(oldSelectedPiece, oldValidTargetSquares) {
  if (oldValidTargetSquares != null) {
    oldValidTargetSquares.forEach((index) => {
      const cell = tbody.querySelector(`[data-index="${index}"]`);
      cell.classList.remove('target_square');
    });
  }
  if (validTargetSquares != null) {
    validTargetSquares.forEach((index) => {
      const cell = tbody.querySelector(`[data-index="${index}"]`);
      cell.classList.add('target_square');
    });
  }
  if (oldSelectedPiece != null) {
    const cell = tbody.querySelector(`[data-index="${oldSelectedPiece}"]`);
    cell.classList.remove('source_square');
  }
  if (selectedPiece != null) {
    const cell = tbody.querySelector(`[data-index="${selectedPiece}"]`);
    cell.classList.add('source_square');
  }
}

function updateSelectedPiece(index) {
  selectedPiece = index;
  validTargetSquares = [];
  pseudoLegalMoves.forEach((move) => {
    if (move.from === index) {
      validTargetSquares.push(move.to);
    }
  });
}
