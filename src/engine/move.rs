use crate::engine::{piece::Pieces, square::Square};

pub const MOVE_TYPE_CASTLE: u16 = 0b0000000000000100;
pub const MOVE_TYPE_EN_PASSANT: u16 = 0b0000000000001000;
pub const MOVE_TYPE_PROMOTION: u16 = 0b0000000000001100;

pub const MOVE_PROMOTION_PIECE_KNIGHT: u16 = 0b0000000000000000;
pub const MOVE_PROMOTION_PIECE_BISHOP: u16 = 0b0000000000000001;
pub const MOVE_PROMOTION_PIECE_ROOK: u16 = 0b0000000000000010;
pub const MOVE_PROMOTION_PIECE_QUEEN: u16 = 0b0000000000000011;

pub const MOVE_CASTLE_SIDE_QS: u16 = 0b0000000000000000;
pub const MOVE_CASTLE_SIDE_KS: u16 = 0b0000000000000001;

pub const MOVE_MASK_PIECE: u16 = 0b0000000000000011;
pub const MOVE_MASK_TYPE: u16 = 0b0000000000001100;
pub const MOVE_MASK_START: u16 = 0b0000001111110000;
pub const MOVE_MASK_END: u16 = 0b1111110000000000;

pub type Move = u16;

pub trait MoveUtils {
    fn get_move_type(&self) -> u16;
    fn get_move_piece(&self) -> u16;
    fn get_move_start(&self) -> u16;
    fn get_move_end(&self) -> u16;
    fn move_to_string(&self) -> String;
    fn new_move(start: u16, end: u16, flags: u16) -> Move;
}

impl MoveUtils for Move {
    #[inline(always)]
    fn get_move_type(&self) -> u16 {
        *self & MOVE_MASK_TYPE
    }

    #[inline(always)]
    fn get_move_piece(&self) -> u16 {
        *self & MOVE_MASK_PIECE
    }

    #[inline(always)]
    fn get_move_start(&self) -> u16 {
        (*self & MOVE_MASK_START) >> 4
    }

    #[inline(always)]
    fn get_move_end(&self) -> u16 {
        (*self & MOVE_MASK_END) >> 10
    }

    fn move_to_string(&self) -> String {
        let start = self.get_move_start();
        let end = self.get_move_end();
        let move_type = self.get_move_type();

        let start_sq = Square::from_usize(start as usize);
        let mut result = start_sq.notation();

        let end_sq = Square::from_usize(end as usize);

        if move_type == MOVE_TYPE_EN_PASSANT {
            let end_r = if end_sq.rank() == 3 { 2 } else { 5 };
            result.push_str(&Square::from_rf(end_r, end_sq.file()).notation());
        } else {
            result.push_str(&end_sq.notation());
        }

        if move_type == MOVE_TYPE_PROMOTION {
            let piece = self.get_move_piece() as usize;
            result.push("nbrq".chars().nth(piece).unwrap());
        }

        result
    }

    #[inline(always)]
    fn new_move(start: u16, end: u16, flags: u16) -> Move {
        (end << 10) | (start << 4) | flags
    }
}

#[derive(Default)]
pub struct UndoInfo {
    pub castling: u8,
    pub fifty_move: usize,
    pub en_passant: Option<Square>,
    pub captured: Option<Pieces>,
}
