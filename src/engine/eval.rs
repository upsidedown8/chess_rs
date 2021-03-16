use crate::engine::board::Board;

const PIECE_VALUE: [i32; 12] = [
    1,    // White Pawn
    3,    // White Knight
    3,    // White Bishop
    5,    // White Rook
    9,    // White Queen
    0,    // White King

    -1,   // Black Pawn
    -3,   // Black Knight
    -3,   // Black Bishop
    -5,   // Black Rook
    -9,   // Black Queen
    0,    // Black King
];

pub struct Evaluator {}

impl Evaluator {
    pub fn eval(&self, board: &Board) -> i32 {
        let mut total = 0;

        for &sq in &board.pieces {
            if let Some(piece) = sq {
                total += PIECE_VALUE[piece.idx()];
            }
        }

        if board.friendly_color().is_white() {
            total
        } else {
            -total
        }
    }
}
