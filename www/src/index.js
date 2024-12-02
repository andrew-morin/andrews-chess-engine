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
let nextLegalGameStates = wasm.get_pseudo_legal_moves(gameState);

wasm
  .convert_game_state_to_squares(gameState)
  .forEach(([color, piece], index) => {
    const rankIndex = Math.floor(index / 8);
    const row = document.querySelector(`#rankIndex${rankIndex}`);

    const cell =
      color === 'Empty'
        ? document.createElement('td')
        : getSquareCell(color, piece);
    cell.dataset.index = index;
    cell.addEventListener('click', getOnClick(index));
    row.appendChild(cell);
  });

let selectedPiece = null;
let validTargetSquares = null;
let isPromotion = false;
let playerColor = 'White';
let gameOver = false;

document.addEventListener('click', () => {
  const oldSelectedPiece = selectedPiece;
  const oldValidTargetSquares = validTargetSquares;
  selectedPiece = null;
  validTargetSquares = null;
  updateCellClasses(oldSelectedPiece, oldValidTargetSquares);
});

function updateBoard(move) {
  const targetCell = document.querySelector(`[data-index="${move.to}"]`);
  if (targetCell.firstChild) {
    targetCell.removeChild(targetCell.firstChild);
  }
  const sourceCell = document.querySelector(`[data-index="${move.from}"]`);
  if (move.promotion_piece) {
    sourceCell.removeChild(sourceCell?.firstChild);
    const img = getPieceImage(gameState.turn, move.promotion_piece);
    targetCell.appendChild(img);
  } else {
    targetCell.appendChild(sourceCell.firstChild);
  }
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
  }
}

function getPieceImage(color, piece) {
  const img = document.createElement('img');
  img.src = SQUARE_IMAGE_MAP[color][piece];
  return img;
}

function getSquareCell(color, piece) {
  const cell = document.createElement('td');
  cell.appendChild(getPieceImage(color, piece));

  return cell;
}

function getOnClick(index) {
  return (event) => {
    if (isPromotion) {
      hidePromotionChoice();
    }
    const [color] = wasm.get_square_at_index(gameState, index);
    const oldValidTargetSquares = validTargetSquares;
    const oldSelectedPiece = selectedPiece;
    let move;
    if (
      gameState.turn == playerColor &&
      selectedPiece != null &&
      validTargetSquares.includes(index)
    ) {
      const nextMoves = nextLegalGameStates.filter((move) => {
        return move.from === selectedPiece && move.to === index;
      });
      if (nextMoves.length > 1) {
        showPromotionChoice(gameState.turn, nextMoves);
      } else {
        move = nextMoves[0];
      }
    } else if (color === 'EMPTY' || selectedPiece === index) {
      selectedPiece = null;
      validTargetSquares = null;
    } else {
      updateSelectedPiece(index);
    }
    performMove(move);
    updateCellClasses(oldSelectedPiece, oldValidTargetSquares, true);
    event.stopPropagation();
  };
}

function performMove(move) {
  if (!move || gameState.turn !== playerColor) {
    return;
  }
  updateBoard(move);
  gameState = wasm.perform_move(gameState, move);
  nextLegalGameStates = wasm.get_pseudo_legal_moves(gameState);
  selectedPiece = null;
  validTargetSquares = null;
  checkForWinLoseDraw();
  performComputerMove();
}

function performComputerMove() {
  if (gameOver || gameState.turn === playerColor) {
    return;
  }
  const { game_state: newGameState, next_move: move } =
    wasm.get_best_engine_move(gameState);
  gameState = newGameState;
  nextLegalGameStates = wasm.get_pseudo_legal_moves(gameState);
  updateBoard(move);
}

function checkForWinLoseDraw() {
  if (gameState.halfmove_counter >= 100) {
    gameOver = true;
    alert("No capture or pawn move in 50 moves. It's a draw! :/");
  } else if (nextLegalGameStates.length === 0) {
    gameOver = true;
    const inCheckReturn = wasm.in_check(gameState);
    const inCheck = inCheckReturn[0];
    if (inCheck) {
      if (gameState.turn === 'White') {
        alert('You lose :(');
      } else {
        alert('You win! :D');
      }
    } else {
      alert("It's a draw! :/");
    }
  }
}

function getPromOnClick(nextMove) {
  return (event) => {
    const oldValidTargetSquares = validTargetSquares;
    const oldSelectedPiece = selectedPiece;
    performMove(nextMove);
    hidePromotionChoice();
    updateCellClasses(oldSelectedPiece, oldValidTargetSquares, true);
    checkForWinLoseDraw();
    event.stopPropagation();
  };
}

function showPromotionChoice(turn, nextMoves) {
  const promDiv = document.querySelector('#promotionPieces');
  nextMoves.forEach((nextMove) => {
    const cell = getSquareCell(turn, nextMove.promotion_piece);
    cell.addEventListener('click', getPromOnClick(nextMove));
    promDiv.appendChild(cell);
  });
  isPromotion = true;
}

function hidePromotionChoice() {
  const promDiv = document.querySelector('#promotionPieces');
  promDiv.replaceChildren();
  isPromotion = false;
}

function updateCellClasses(
  oldSelectedPiece,
  oldValidTargetSquares,
  checkForCheck = false
) {
  if (oldValidTargetSquares != null) {
    oldValidTargetSquares.forEach((index) => {
      const cell = document.querySelector(`[data-index="${index}"]`);
      cell.classList.remove('target_square');
    });
  }
  if (playerColor == gameState.turn && validTargetSquares != null) {
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
  if (checkForCheck) {
    const checkedKing = document.querySelector('.king_check');
    const inCheckReturn = wasm.in_check(gameState);
    const inCheck = inCheckReturn[0];
    const kingIndex = inCheckReturn[1];
    if (checkedKing) {
      checkedKing.classList.remove('king_check');
    } else if (inCheck) {
      const cell = document.querySelector(`[data-index="${kingIndex}"]`);
      cell.classList.add('king_check');
    }
  }
}

function updateSelectedPiece(index) {
  selectedPiece = index;
  validTargetSquares = [];
  nextLegalGameStates.forEach((move) => {
    if (move.from === index) {
      validTargetSquares.push(move.to);
    }
  });
}
