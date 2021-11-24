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
let nextLegalGameStates = wasm.get_next_legal_game_states(gameState);

wasm.convert_game_state_to_squares(gameState).forEach(([color, piece], index) => {
  const rankIndex = Math.floor(index / 8);
  const row = document.querySelector(`#rankIndex${rankIndex}`);

  const cell = color === 'Empty' ? document.createElement('td') : getSquareCell(color, piece);
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
  if (move.castle) {
    let rookFrom;
    let rookTo;
    if (move.to === 2) {
      rookFrom = 0;
      rookTo = 3;
    } else if (move.to === 6) {
      rookFrom = 7;
      rookTo = 5;
    } else if (move.to === 58) {
      rookFrom = 56;
      rookTo = 59;
    } else if (move.to === 62) {
      rookFrom = 63;
      rookTo = 61;
    }
    const sourceRookCell = document.querySelector(`[data-index="${rookFrom}"]`);
    const targetRookCell = document.querySelector(`[data-index="${rookTo}"]`);
    targetRookCell.appendChild(sourceRookCell.firstChild);
    // sourceCell.removeChild(sourceCell.firstChild);
  }
}

let selectedPiece = null;
let validTargetSquares = null;

document.addEventListener('click', () => {
  const oldSelectedPiece = selectedPiece;
  const oldValidTargetSquares = validTargetSquares;
  selectedPiece = null;
  validTargetSquares = null;
  updateCellClasses(oldSelectedPiece, oldValidTargetSquares);
});

function updateGameState(_gameState) {
  gameState = _gameState;
  nextLegalGameStates = wasm.get_next_legal_game_states(_gameState);
}

function getPieceImage(color, piece) {
  return SQUARE_IMAGE_MAP[color][piece];
}

function getSquareCell(color, piece) {
  const cell = document.createElement('td');
  const img = document.createElement('img');
  img.src = getPieceImage(color, piece);
  cell.appendChild(img);

  return cell;
}

function getOnClick(index) {
  return (event) => {
    const [color] = wasm.get_square_at_index(gameState, index);
    const oldValidTargetSquares = validTargetSquares;
    const oldSelectedPiece = selectedPiece;
    if (selectedPiece != null && validTargetSquares.includes(index)) {
      const nextGameState = nextLegalGameStates.find(
        (gs) => {
          const { move_list: moveList } = gs;
          const lastMove = moveList[moveList.length - 1];
          return lastMove.from === selectedPiece && lastMove.to === index;
        },
      );
      const { move_list: moveList } = nextGameState;
      const move = moveList[moveList.length - 1];
      gameState = wasm.perform_move(gameState, move);
      updateGameState(gameState);
      updateBoard(move);
      selectedPiece = null;
      validTargetSquares = null;
    } else if (color === 'EMPTY' || selectedPiece === index) {
      selectedPiece = null;
      validTargetSquares = null;
    } else {
      updateSelectedPiece(index);
    }
    updateCellClasses(oldSelectedPiece, oldValidTargetSquares, true);
    event.stopPropagation();
  };
}

function updateCellClasses(oldSelectedPiece, oldValidTargetSquares, checkForCheck) {
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
  const checkedKing = document.querySelector('.king_check');
  if (checkedKing) {
    checkedKing.classList.remove('king_check');
  }
  if (checkForCheck) {
    const inCheckReturn = wasm.in_check(gameState);
    const inCheck = inCheckReturn[0];
    const kingIndex = inCheckReturn[1];
    if (inCheck) {
      const cell = document.querySelector(`[data-index="${kingIndex}"]`);
      cell.classList.add('king_check');
    }
  }
}

function updateSelectedPiece(index) {
  selectedPiece = index;
  validTargetSquares = [];
  nextLegalGameStates.forEach((gs) => {
    const lastMove = gs.move_list[gs.move_list.length - 1];
    if (lastMove.from === index) {
      validTargetSquares.push(lastMove.to);
    }
  });
}
