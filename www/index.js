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

let gameState = wasm.get_initial_game_state();
let pseudoLegalMoves = wasm.get_pseudo_legal_moves(gameState);

gameState.board.forEach((square, index) => {
  const rankIndex = Math.floor(index / 8);
  const row = document.querySelector(`#rankIndex${rankIndex}`);

  const cell = square.empty ? document.createElement('td') : getSquareCell(square);
  cell.dataset.index = index;
  cell.addEventListener('click', getOnClick(index));
  row.appendChild(cell);
});

function updateBoard(move) {
  const targetCell = document.querySelector(`[data-index="${move.to}"]`);
  if (targetCell.firstChild) {
    targetCell.removeChild(targetCell.firstChild);
  }
  const sourceCell = document.querySelector(`[data-index="${move.from}"]`);
  targetCell.appendChild(sourceCell.firstChild);
}

let selectedPiece = null;
let validTargetSquares = null;

document.addEventListener('click', () => {
  const oldSelectedPiece = selectedPiece;
  const oldValidTargetSquares = validTargetSquares;
  selectedPiece = null;
  validTargetSquares = null;
  updateValidCells(oldSelectedPiece, oldValidTargetSquares);
});

function updateGameState(_gameState) {
  gameState = _gameState;
  pseudoLegalMoves = wasm.get_pseudo_legal_moves(_gameState);
  console.log(pseudoLegalMoves);
}

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

function getOnClick(index) {
  return (event) => {
    const square = gameState.board[index];
    const oldValidTargetSquares = validTargetSquares;
    const oldSelectedPiece = selectedPiece;
    if (selectedPiece != null && validTargetSquares.includes(index)) {
      const move = pseudoLegalMoves.find(
        (_move) => _move.from === selectedPiece && _move.to === index,
      );
      gameState = wasm.perform_move(gameState, move);
      updateGameState(gameState);
      updateBoard(move);
      selectedPiece = null;
      validTargetSquares = null;
    } else if (square.empty || selectedPiece === index) {
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
      const cell = document.querySelector(`[data-index="${index}"]`);
      cell.classList.remove('target_square');
    });
  }
  if (validTargetSquares != null) {
    validTargetSquares.forEach((index) => {
      const cell = document.querySelector(`[data-index="${index}"]`);
      cell.classList.add('target_square');
    });
  }
  if (oldSelectedPiece != null) {
    const cell = document.querySelector(`[data-index="${oldSelectedPiece}"]`);
    cell.classList.remove('source_square');
  }
  if (selectedPiece != null) {
    const cell = document.querySelector(`[data-index="${selectedPiece}"]`);
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
