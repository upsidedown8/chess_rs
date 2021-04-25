use crate::engine::board::Board;

use super::piece::{Color, Pieces};

const PIECE_VALUE: [i32; 12] = [
    100,  // White Pawn
    315,  // White Knight
    325,  // White Bishop
    500,  // White Rook
    900,  // White Queen
    0,    // White King

    -100, // Black Pawn
    -315, // Black Knight
    -325, // Black Bishop
    -500, // Black Rook
    -900, // Black Queen
    0,    // Black King
];

const PAWN_SQ_VALUE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    50, 50, 50, 50, 50, 50, 50, 50,
    10, 10, 20, 30, 30, 20, 10, 10,
     5,  5, 10, 25, 25, 10,  5,  5,
     0,  0,  0, 20, 20,  0,  0,  0,
     5, -5,-10,  0,  0,-10, -5,  5,
     5, 10, 10,-20,-20, 10, 10,  5,
     0,  0,  0,  0,  0,  0,  0,  0
];

const KNIGHT_SQ_VALUE: [i32; 64] = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

const BISHOP_SQ_VALUE: [i32; 64] = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

const ROOK_SQ_VALUE: [i32; 64] = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

const QUEEN_SQ_VALUE: [i32; 64] = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

const KING_SQ_VALUE: [i32; 64] = [
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -30,-40,-40,-50,-50,-40,-40,-30,
    -20,-30,-30,-40,-40,-30,-30,-20,
    -10,-20,-20,-20,-20,-20,-20,-10,
     20, 20,  0,  0,  0,  0, 20, 20,
     20, 30, 10,  0,  0, 10, 30, 20
];

const SQ_VALUE: [&[i32; 64]; 6] = [
    &PAWN_SQ_VALUE,
    &KNIGHT_SQ_VALUE,
    &BISHOP_SQ_VALUE,
    &ROOK_SQ_VALUE,
    &QUEEN_SQ_VALUE,
    &KING_SQ_VALUE,
];

#[derive(Default)]
pub struct Evaluator {
    score: i32
}

impl Evaluator {
    pub fn en_passant_diff(start: usize, en_passant_sq: usize, end: usize, friendly_pawn: Pieces) -> i32 {
        let mut diff = 0;
        
        // remove enemy pawn
        let enemy_pawn = Pieces::pawn(friendly_pawn.color().enemy());
        diff -= Evaluator::piece_value(enemy_pawn);
        diff -= Evaluator::sq_value(enemy_pawn, end);

        // move friendly pawn        
        diff -= Evaluator::sq_value(friendly_pawn, start);
        diff += Evaluator::sq_value(friendly_pawn, en_passant_sq);

        diff
    }
    pub fn castle_diff(king_start: usize, king_end: usize, rook_start: usize, rook_end: usize, color: Color) -> i32 {
        let mut diff = 0;
        
        // move king
        let king = Pieces::king(color);
        diff -= Evaluator::sq_value(king, king_start);
        diff += Evaluator::sq_value(king, king_end);

        // move rook
        let rook = Pieces::rook(color);
        diff -= Evaluator::sq_value(rook, rook_start);
        diff += Evaluator::sq_value(rook, rook_end);

        diff
    }
    pub fn promotion_diff(pawn_start: usize, promotion_end: usize, promotion_piece: Pieces, captured_piece: Option<Pieces>, color: Color) -> i32 {
        let mut diff = 0;
        
        // promote pawn
        let friendly_pawn = Pieces::pawn(color);
        diff -= Evaluator::piece_value(friendly_pawn);
        diff -= Evaluator::sq_value(friendly_pawn, pawn_start);
        diff += Evaluator::piece_value(promotion_piece);
        diff += Evaluator::sq_value(promotion_piece, promotion_end);

        // capture piece
        if let Some(captured_piece) = captured_piece {
            diff -= Evaluator::piece_value(captured_piece);
            diff -= Evaluator::sq_value(captured_piece, promotion_end);
        }

        diff
    }
    pub fn standard_diff(piece_start: usize, piece_end: usize, piece: Pieces, captured_piece: Option<Pieces>) -> i32 {
        let mut diff = 0;

        // move piece
        diff -= Evaluator::sq_value(piece, piece_start);
        diff += Evaluator::sq_value(piece, piece_end);

        // capture piece
        if let Some(captured_piece) = captured_piece {
            diff -= Evaluator::piece_value(captured_piece);
            diff -= Evaluator::sq_value(captured_piece, piece_end);
        }

        diff
    }

    fn sq_value(piece: Pieces, sq: usize) -> i32 {
        let piece_idx = piece.idx() % 6;
        SQ_VALUE[piece_idx][sq] * if piece.color().is_white() { 1 } else { -1 }
    }

    pub fn piece_value(piece: Pieces) -> i32 {
        PIECE_VALUE[piece.idx()]
    }
    
    pub fn update_score(&mut self, diff: i32) {
        self.score += diff;
    }
    pub fn init_score(&mut self, board: &Board) {
        self.score = 0;

        for sq in 0..64  {
            if let Some(piece) = board.pieces[sq] {
                self.score +=
                    Evaluator::piece_value(piece) +
                    Evaluator::sq_value(piece, sq);
            }
        }
    }

    pub fn score(&self, color: Color) -> i32 {
        if color.is_white() {
            self.score
        } else {
            -self.score
        }
    }
}
