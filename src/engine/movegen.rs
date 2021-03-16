use std::fmt::{Display, Formatter, Result};

use crate::engine::bitboard::BitBoardUtils;
use crate::engine::board::Board;
use crate::engine::piece::{Color, Pieces};
use crate::engine::r#move::*;
use crate::engine::square::Square;

use super::bitboard::FULL_BB;

trait PlayerTrait {
    fn color() -> Color;
    fn enemy() -> Color;
    fn is_white() -> bool;
    fn opposite_back_rank() -> Ranks;
    fn en_passant_rank() -> Ranks;
    fn capture_offset(is_left_capture: bool) -> i16;
    fn forward_offset() -> i16;
}

struct WhitePlayer {}
impl PlayerTrait for WhitePlayer {
    #[inline(always)]
    fn color() -> Color {
        Color::White
    }

    #[inline(always)]
    fn enemy() -> Color {
        Color::Black
    }

    #[inline(always)]
    fn is_white() -> bool {
        true
    }

    #[inline(always)]
    fn opposite_back_rank() -> Ranks {
        Ranks::Eight
    }

    #[inline(always)]
    fn en_passant_rank() -> Ranks {
        Ranks::Three
    }

    #[inline(always)]
    fn capture_offset(is_left_capture: bool) -> i16 {
        if is_left_capture {
            9
        } else {
            7
        }
    }

    #[inline(always)]
    fn forward_offset() -> i16 {
        8
    }
}

struct BlackPlayer {}
impl PlayerTrait for BlackPlayer {
    #[inline(always)]
    fn color() -> Color {
        Color::Black
    }

    #[inline(always)]
    fn enemy() -> Color {
        Color::White
    }
    
    #[inline(always)]
    fn is_white() -> bool {
        false
    }

    #[inline(always)]
    fn opposite_back_rank() -> Ranks {
        Ranks::One
    }

    #[inline(always)]
    fn en_passant_rank() -> Ranks {
        Ranks::Six
    }

    #[inline(always)]
    fn capture_offset(is_left_capture: bool) -> i16 {
        if is_left_capture {
            -7
        } else {
            -9
        }
    }

    #[inline(always)]
    fn forward_offset() -> i16 {
        -8
    }
}

trait CaptureSideTrait {
    fn is_left() -> bool;
}

struct LeftCapture {}
impl CaptureSideTrait for LeftCapture {
    #[inline(always)]
    fn is_left() -> bool {
        true
    }
}

struct RightCapture {}
impl CaptureSideTrait for RightCapture {
    #[inline(always)]
    fn is_left() -> bool {
        false
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Ranks {
    One = 0b0000_0001,
    Two = 0b0000_0010,
    Three = 0b0000_0100,
    Four = 0b0000_1000,
    Five = 0b0001_0000,
    Six = 0b0010_0000,
    Seven = 0b0100_0000,
    Eight = 0b1000_0000,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Files {
    A = 0b0000_0001,
    B = 0b0000_0010,
    C = 0b0000_0100,
    D = 0b0000_1000,
    E = 0b0001_0000,
    F = 0b0010_0000,
    G = 0b0100_0000,
    H = 0b1000_0000,
}

impl Files {
    #[inline(always)]
    fn idx(&self) -> usize {
        *self as usize
    }
}

pub struct MoveList {
    moves: Vec<Move>,
}

impl MoveList {
    #[inline(always)]
    pub fn add_move_with_flags(&mut self, start: usize, end: usize, flags: u16) {
        self.moves
            .push(Move::new_move(start as u16, end as u16, flags));
    }

    #[inline(always)]
    pub fn add_move(&mut self, start: usize, end: usize) {
        self.add_move_with_flags(start, end, 0);
    }

    #[inline(always)]
    pub fn add_promotion(&mut self, start: usize, end: usize) {
        self.add_move_with_flags(start, end, MOVE_TYPE_PROMOTION | MOVE_PROMOTION_PIECE_QUEEN);
        self.add_move_with_flags(start, end, MOVE_TYPE_PROMOTION | MOVE_PROMOTION_PIECE_ROOK);
        self.add_move_with_flags(
            start,
            end,
            MOVE_TYPE_PROMOTION | MOVE_PROMOTION_PIECE_BISHOP,
        );
        self.add_move_with_flags(
            start,
            end,
            MOVE_TYPE_PROMOTION | MOVE_PROMOTION_PIECE_KNIGHT,
        );
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.moves.len()
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.moves.clear();
    }

    pub fn new() -> MoveList {
        let mut result = MoveList { moves: Vec::new() };
        result.moves.reserve(256);
        result
    }

    #[inline(always)]
    pub fn at(&self, idx: usize) -> Move {
        self.moves[idx]
    }
}

impl Display for MoveList {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut result = String::new();

        for (i, &my_move) in self.moves.iter().enumerate() {
            result.push_str(&format!(
                "{}: {}\n",
                i+1,
                my_move.move_to_string()
            ));
        }

        write!(f, "{}", result)
    }
}

pub struct MoveGenerator {
    rook_masks: [u64; 64],
    bishop_masks: [u64; 64],

    rook_magic_shifts: [usize; 64],
    bishop_magic_shifts: [usize; 64],

    rook_moves: Box<[[u64; 4096]]>,
    bishop_moves: Box<[[u64; 4096]]>,

    knight_moves: [u64; 64],
    king_moves: [u64; 64],

    pawn_attacks: [[u64; 64]; 2],

    slider_range: [[u64; 64]; 64],

    ranks: [u64; 256],
    files: [u64; 256],

    not_ranks: [u64; 256],
    not_files: [u64; 256],
}

impl MoveGenerator {
    /* -------------------------------------------------------------------------- */
    /*                                    Setup                                   */
    /* -------------------------------------------------------------------------- */
    fn gen_rook_mask(&self, start: Square) -> u64 {
        let rank = start.rank() as i16;
        let file = start.file() as i16;
        let mut result = 0;

        let mut r = rank - 1;
        let mut f = file;

        while r >= 1 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            r -= 1;
        }

        r = rank + 1;
        while r <= 6 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            r += 1;
        }

        r = rank;
        f = file - 1;
        while f >= 1 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            f -= 1;
        }

        f = file + 1;
        while f <= 6 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            f += 1;
        }

        result
    }
    fn gen_bishop_mask(&self, start: Square) -> u64 {
        let rank = start.rank() as i16;
        let file = start.file() as i16;
        let mut result = 0;

        let mut r = rank - 1;
        let mut f = file - 1;
        while r >= 1 && f >= 1 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            r -= 1;
            f -= 1;
        }

        r = rank + 1;
        f = file - 1;
        while r <= 6 && f >= 1 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            r += 1;
            f -= 1;
        }

        r = rank - 1;
        f = file + 1;
        while r >= 1 && f <= 6 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            r -= 1;
            f += 1;
        }

        r = rank + 1;
        f = file + 1;
        while r <= 6 && f <= 6 {
            result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            r += 1;
            f += 1;
        }

        result
    }

    fn gen_rook_moves(&self, start: Square, occupancy: u64) -> u64 {
        let rank = start.rank() as i16;
        let file = start.file() as i16;
        let mut result = 0;

        let mut r = rank - 1;
        let mut f = file;

        while r >= 0 {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r -= 1;
        }

        r = rank + 1;
        while r <= 7 {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r += 1;
        }

        r = rank;
        f = file - 1;
        while f >= 0 {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            f -= 1;
        }

        f = file + 1;
        while f <= 7 {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            f += 1;
        }

        result
    }
    fn gen_bishop_moves(&self, start: Square, occupancy: u64) -> u64 {
        let rank = start.rank() as i16;
        let file = start.file() as i16;
        let mut result = 0;

        let mut r = rank - 1;
        let mut f = file - 1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r -= 1;
            f -= 1;
        }

        r = rank + 1;
        f = file - 1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r += 1;
            f -= 1;
        }

        r = rank - 1;
        f = file + 1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r -= 1;
            f += 1;
        }

        r = rank + 1;
        f = file + 1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize).sq();
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r += 1;
            f += 1;
        }

        result
    }
    fn gen_king_moves(&self, start: Square) -> u64 {
        let mut result = 0;

        let rank = start.rank() as i16;
        let file = start.file() as i16;

        const MOVE_VECTORS: [[i16; 2]; 8] = [
            [-1, -1],
            [-1, 0],
            [-1, 1],
            [0, -1],
            [0, 1],
            [1, -1],
            [1, 0],
            [1, 1],
        ];

        for &vector in &MOVE_VECTORS {
            let f = file + vector[0];
            let r = rank + vector[1];
            if Square::valid_rf(r, f) {
                result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            }
        }

        result
    }
    fn gen_knight_moves(&self, start: Square) -> u64 {
        let mut result = 0;

        let rank = start.rank() as i16;
        let file = start.file() as i16;

        const MOVE_VECTORS: [[i16; 2]; 8] = [
            [-1, 2],
            [-2, 1],
            [1, 2],
            [2, 1],
            [-1, -2],
            [-2, -1],
            [1, -2],
            [2, -1],
        ];

        for &vector in &MOVE_VECTORS {
            let f = file + vector[0];
            let r = rank + vector[1];
            if Square::valid_rf(r, f) {
                result.set_bit(Square::from_rf(r as usize, f as usize).sq());
            }
        }

        result
    }

    fn idx_to_u64(&self, idx: usize, mut mask: u64) -> u64 {
        let mut result = 0;

        let mut i = 0;
        while idx != 0 && mask != 0 {
            let pos = mask.pop_lsb();
            if idx & (1 << i) != 0 {
                result.set_bit(pos);
            }
            i += 1;
        }

        result
    }

    fn init(&mut self) {
        // init ranks & files
        for i in 0..8 {
            self.files[1 << i] = 0x0101010101010101 << i;
            self.ranks[1 << i] = 0xff << (8 * (7 - i));
        }
        for i in 0..256 {
            for j in 0..8 {
                if i & (1 << j) != 0 {
                    self.files[i] |= self.files[1 << j];
                    self.ranks[i] |= self.ranks[1 << j];
                }
            }
            self.not_files[i] = !self.files[i];
            self.not_ranks[i] = !self.ranks[i];
        }

        // masks, shifts and move tables
        for i in 0..64 {
            let sq = Square::from_usize(i);

            // rook & bishop masks
            self.rook_masks[i] = self.gen_rook_mask(sq);
            self.bishop_masks[i] = self.gen_bishop_mask(sq);

            // rook & bishop shifts
            self.rook_magic_shifts[i] = 64 - self.rook_masks[i].count_1s();
            self.bishop_magic_shifts[i] = 64 - self.bishop_masks[i].count_1s();

            // rook & bishop move tables
            for idx in 0..(1 << self.rook_masks[i].count_1s()) {
                let indexed_mask = self.idx_to_u64(idx, self.rook_masks[i]);
                let key = u64::wrapping_mul(ROOK_MAGICS[i], indexed_mask) >> self.rook_magic_shifts[i];
                self.rook_moves[i][key as usize] = self.gen_rook_moves(sq, indexed_mask);
            }
            for idx in 0..(1 << self.bishop_masks[i].count_1s()) {
                let indexed_mask = self.idx_to_u64(idx, self.bishop_masks[i]);
                let key = u64::wrapping_mul(BISHOP_MAGICS[i], indexed_mask) >> self.bishop_magic_shifts[i];
                self.bishop_moves[i][key as usize] = self.gen_bishop_moves(sq, indexed_mask);
            }

            // knight moves
            self.knight_moves[i] = self.gen_knight_moves(sq);

            // king moves
            self.king_moves[i] = self.gen_king_moves(sq);

            // pawn attacks
            if sq.rank() != 0 {
                if sq.file() != 7 {
                    self.pawn_attacks[Color::White.idx()][i].set_bit(i - 7);
                }
                if sq.file() != 0 {
                    self.pawn_attacks[Color::White.idx()][i].set_bit(i - 9);
                }
            }
            if sq.rank() != 7 {
                if sq.file() != 7 {
                    self.pawn_attacks[Color::Black.idx()][i].set_bit(i + 9);
                }
                if sq.file() != 0 {
                    self.pawn_attacks[Color::Black.idx()][i].set_bit(i + 7);
                }
            }
        }

        // slider range
        for start in 0..64 {
            for end in 0..64 {
                let min_sq = Square::from_usize(std::cmp::min(start, end));
                let max_sq = Square::from_usize(std::cmp::max(start, end));

                self.slider_range[start][end] = 0;

                // same rank
                if min_sq.rank() == max_sq.rank() {
                    for f in min_sq.file()+1..max_sq.file() {
                        self.slider_range[start][end].set_bit(Square::from_rf(min_sq.rank(), f).sq());
                    }
                }
                
                // same file
                else if min_sq.file() == max_sq.file() {
                    for r in min_sq.rank()+1..max_sq.rank() {
                        self.slider_range[start][end].set_bit(Square::from_rf(r, min_sq.file()).sq());
                    }
                }
                
                else {
                    let rank_diff = max_sq.rank() as i16 - min_sq.rank() as i16;
                    let file_diff = max_sq.file() as i16 - min_sq.file() as i16;
                    
                    // diagonal
                    if rank_diff.abs() == file_diff.abs() {
                        // negative gradient
                        if rank_diff == file_diff {
                            let mut r = min_sq.rank() + 1;
                            let mut f = min_sq.file() + 1;
                            while r < max_sq.rank() && f < max_sq.file() {
                                self.slider_range[start][end].set_bit(Square::from_rf(r, f).sq());
                                r += 1;
                                f += 1;
                            }
                        }
                        
                        // positive gradient
                        else {
                            let mut r = min_sq.rank() + 1;
                            let mut f = min_sq.file() - 1;
                            while r < max_sq.rank() && f > max_sq.file() {
                                self.slider_range[start][end].set_bit(Square::from_rf(r, f).sq());
                                r += 1;
                                f -= 1;
                            }

                        }
                    }
                }
            }
        }
    }

    pub fn new() -> MoveGenerator {
        let mut result = MoveGenerator {
            rook_masks: [0; 64],
            bishop_masks: [0; 64],

            rook_magic_shifts: [0; 64],
            bishop_magic_shifts: [0; 64],

            rook_moves: vec![[0; 4096]; 64].into_boxed_slice(),
            bishop_moves: vec![[0; 4096]; 64].into_boxed_slice(),

            knight_moves: [0; 64],
            king_moves: [0; 64],

            pawn_attacks: [[0; 64]; 2],

            slider_range: [[0; 64]; 64],

            ranks: [0; 256],
            files: [0; 256],

            not_ranks: [0; 256],
            not_files: [0; 256],
        };

        result.init();

        result
    }

    /* -------------------------------------------------------------------------- */
    /*                               Move Generation                              */
    /* -------------------------------------------------------------------------- */
    #[inline(always)]
    fn magic_bishop_moves(&self, sq: usize, mut occupancy: u64) -> u64 {
        occupancy &= self.bishop_masks[sq];
        let idx = u64::wrapping_mul(BISHOP_MAGICS[sq], occupancy) >> self.bishop_magic_shifts[sq];
        self.bishop_moves[sq][idx as usize]
    }
    #[inline(always)]
    fn magic_rook_moves(&self, sq: usize, mut occupancy: u64) -> u64 {
        occupancy &= self.rook_masks[sq];
        let idx = u64::wrapping_mul(ROOK_MAGICS[sq], occupancy) >> self.rook_magic_shifts[sq];
        self.rook_moves[sq][idx as usize]
    }
    #[inline(always)]
    fn magic_queen_moves(&self, sq: usize, occupancy: u64) -> u64 {
        self.magic_bishop_moves(sq, occupancy) | self.magic_rook_moves(sq, occupancy)
    }

    #[inline(always)]
    fn find_enemy_attackers<P: PlayerTrait>(&self, sq: usize, board: &Board, occupancy: u64) -> u64 {
        let bishop_queen = board.get_bb(Pieces::bishop(P::enemy()))
            | board.get_bb(Pieces::queen(P::enemy()));
        let rook_queen = board.get_bb(Pieces::rook(P::enemy()))
            | board.get_bb(Pieces::queen(P::enemy()));

        let mut attacking_pieces = (self.magic_bishop_moves(sq, occupancy) & bishop_queen)
            | (self.magic_rook_moves(sq, occupancy) & rook_queen);

        // knights
        attacking_pieces |= (self.knight_moves[sq]
            & board.get_bb(Pieces::knight(P::enemy())))
            | (self.king_moves[sq] & board.get_bb(Pieces::king(P::enemy())))
            | (self.pawn_attacks[P::color().idx()][sq]
                & board.get_bb(Pieces::pawn(P::enemy())));

        attacking_pieces
    }
    #[inline(always)]
    fn is_sq_under_attack<P: PlayerTrait>(&self, sq: usize, board: &Board, occupancy: u64) -> bool {
        if self.knight_moves[sq] & board.get_bb(Pieces::knight(P::enemy())) != 0
            || self.king_moves[sq] & board.get_bb(Pieces::king(P::enemy())) != 0
            || self.pawn_attacks[P::color().idx()][sq]
                & board.get_bb(Pieces::pawn(P::enemy()))
                != 0
        {
            return true;
        }

        let bishop_queen = board.get_bb(Pieces::bishop(P::enemy()))
            | board.get_bb(Pieces::queen(P::enemy()));
        if self.magic_bishop_moves(sq, occupancy) & bishop_queen != 0 {
            return true;
        }

        let rook_queen = board.get_bb(Pieces::rook(P::enemy()))
            | board.get_bb(Pieces::queen(P::enemy()));
        self.magic_rook_moves(sq, occupancy) & rook_queen != 0
    }

    #[inline(always)]
    fn validate_en_passant<P: PlayerTrait>(
        &self,
        board: &mut Board,
        king_pos: usize,
        start: usize,
        end: usize,
        mut occupancy: u64,
    ) -> bool {
        let en_passant_sq = board.en_passant.unwrap().sq();

        // pawn bitboards
        board
            .get_bb_mut(Pieces::pawn(P::enemy()))
            .clear_bit(end);
        board
            .get_bb_mut(Pieces::pawn(P::color()))
            .clear_bit(start)
            .set_bit(en_passant_sq);

        // combined bitboards
        board
            .get_combined_bb_mut(P::enemy())
            .clear_bit(end);
        board
            .get_combined_bb_mut(P::color())
            .clear_bit(start)
            .set_bit(en_passant_sq);

        occupancy
            .clear_bit(start)
            .set_bit(en_passant_sq)
            .clear_bit(end);

        let result = !self.is_sq_under_attack::<P>(king_pos, board, occupancy);

        // pawn bitboards
        board
            .get_bb_mut(Pieces::pawn(P::enemy()))
            .set_bit(end);
        board
            .get_bb_mut(Pieces::pawn(P::color()))
            .set_bit(start)
            .clear_bit(en_passant_sq);

        // combined bitboards
        board.get_combined_bb_mut(P::enemy()).set_bit(end);
        board
            .get_combined_bb_mut(P::color())
            .set_bit(start)
            .clear_bit(en_passant_sq);

        result
    }

    fn add_pawn_captures<P: PlayerTrait, C: CaptureSideTrait>(
        &self,
        move_list: &mut MoveList,
        board: &mut Board,
        occupancy: u64,
        pinned: u64,
        legal_captures: u64,
        king_pos: usize,
    ) {
        let enemy_bb = board.get_combined_bb(P::enemy());
        let pawns_bb = board.get_bb(Pieces::pawn(P::color())) & !pinned;
        let offset = P::capture_offset(C::is_left());

        let back_rank = P::opposite_back_rank();
        let excluded_file = if C::is_left() { Files::A } else { Files::H };
        let file_mask = self.not_files[excluded_file as usize];
        let mut captures = if P::is_white() {
            (pawns_bb & file_mask) >> (if C::is_left() { 9 } else { 7 })
        } else {
            (pawns_bb & file_mask) << (if C::is_left() { 7 } else { 9 })
        };

        if board.en_passant.is_some() {
            let en_passant = board.en_passant.unwrap().sq();
            let start = (en_passant as i16 + offset) as usize;
            let en_passant_end =
                (en_passant as i16 + P::forward_offset()) as usize;
            if captures.is_bit_set(en_passant)
                && self.validate_en_passant::<P>(board, king_pos, start, en_passant_end, occupancy)
            {
                move_list.add_move_with_flags(start, en_passant_end, MOVE_TYPE_EN_PASSANT);
            }
        }

        captures &= legal_captures & enemy_bb;

        let mut non_promotion_caps = captures & self.not_ranks[back_rank as usize];
        while non_promotion_caps != 0 {
            let end = non_promotion_caps.pop_lsb() as i16;
            move_list.add_move((offset + end) as usize, end as usize);
        }

        let mut promotion_caps = captures & self.ranks[back_rank as usize];
        while promotion_caps != 0 {
            let end = promotion_caps.pop_lsb() as i16;
            move_list.add_promotion((offset + end) as usize, end as usize);
        }
    }
    fn add_pawn_pushes<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
        occupancy: u64,
        pinned: u64,
        blockers: u64,
    ) {
        let pawns_bb = board.get_bb(Pieces::pawn(P::color())) & !pinned;
        let offset: i16 = P::forward_offset();
        let double_offset = offset * 2;
        let back_rank = P::opposite_back_rank();
        let en_passant_rank = P::en_passant_rank();

        let mut pawn_single_moves = if P::is_white() {
            pawns_bb >> 8
        } else {
            pawns_bb << 8
        } & !occupancy;

        let mut non_promotion_moves =
            pawn_single_moves & blockers & self.not_ranks[back_rank as usize];
        while non_promotion_moves != 0 {
            let end = non_promotion_moves.pop_lsb() as i16;
            move_list.add_move((offset + end) as usize, end as usize);
        }

        let mut promotion_moves = pawn_single_moves & blockers & self.ranks[back_rank as usize];
        while promotion_moves != 0 {
            let end = promotion_moves.pop_lsb() as i16;
            move_list.add_promotion((offset + end) as usize, end as usize);
        }

        pawn_single_moves &= self.ranks[en_passant_rank as usize];
        let mut pawn_double_moves = if P::is_white() {
            pawn_single_moves >> 8
        } else {
            pawn_single_moves << 8
        } & !occupancy
            & blockers;
        while pawn_double_moves != 0 {
            let end = pawn_double_moves.pop_lsb() as i16;
            move_list.add_move((double_offset + end) as usize, end as usize);
        }
    }

    /* -------------------------------------------------------------------------- */
    /*                              Non-Pinned Pieces                             */
    /* -------------------------------------------------------------------------- */
    #[inline(always)]
    fn add_pawn_moves<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &mut Board,
        occupancy: u64,
        pinned: u64,
        legal_captures: u64,
        blockers: u64,
        king_pos: usize,
    ) {
        self.add_pawn_captures::<P, LeftCapture>(
            move_list,
            board,
            occupancy,
            pinned,
            legal_captures,
            king_pos,
        );
        self.add_pawn_captures::<P, RightCapture>(
            move_list,
            board,
            occupancy,
            pinned,
            legal_captures,
            king_pos,
        );
        self.add_pawn_pushes::<P>(move_list, board, occupancy, pinned, blockers);
    }
    #[inline(always)]
    fn add_knight_moves<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
        pinned: u64,
        move_mask: u64,
    ) {
        let mut knights_bb = board.get_bb(Pieces::knight(P::color())) & !pinned;
        let mask = move_mask & !board.get_combined_bb(P::color());

        while knights_bb != 0 {
            let start = knights_bb.pop_lsb();
            let mut knight_moves = self.knight_moves[start] & mask;

            while knight_moves != 0 {
                move_list.add_move(start, knight_moves.pop_lsb());
            }
        }
    }
    #[inline(always)]
    fn add_bishop_moves<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
        occupancy: u64,
        pinned: u64,
        move_mask: u64,
    ) {
        let mut bishops_bb = board.get_bb(Pieces::bishop(P::color())) & !pinned;
        let mask = move_mask & !board.get_combined_bb(P::color());

        while bishops_bb != 0 {
            let start = bishops_bb.pop_lsb();
            let mut bishop_moves = self.magic_bishop_moves(start, occupancy) & mask;

            while bishop_moves != 0 {
                move_list.add_move(start, bishop_moves.pop_lsb());
            }
        }
    }
    #[inline(always)]
    fn add_rook_moves<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
        occupancy: u64,
        pinned: u64,
        move_mask: u64,
    ) {
        let mut rooks_bb = board.get_bb(Pieces::rook(P::color())) & !pinned;
        let mask = move_mask & !board.get_combined_bb(P::color());

        while rooks_bb != 0 {
            let start = rooks_bb.pop_lsb();
            let mut rook_moves = self.magic_rook_moves(start, occupancy) & mask;

            while rook_moves != 0 {
                move_list.add_move(start, rook_moves.pop_lsb());
            }
        }
    }
    #[inline(always)]
    fn add_queen_moves<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &Board,
        occupancy: u64,
        pinned: u64,
        move_mask: u64,
    ) {
        let mut queens_bb = board.get_bb(Pieces::queen(P::color())) & !pinned;
        let mask = move_mask & !board.get_combined_bb(P::color());

        while queens_bb != 0 {
            let start = queens_bb.pop_lsb();
            let mut queen_moves = self.magic_queen_moves(start, occupancy) & mask;

            while queen_moves != 0 {
                move_list.add_move(start, queen_moves.pop_lsb());
            }
        }
    }
    #[inline(always)]
    fn add_king_moves<P: PlayerTrait>(&self, move_list: &mut MoveList, board: &Board, mut occupancy: u64) {
        let king_bb = board.get_bb(Pieces::king(P::color()));
        let start = king_bb.lsb_idx();
        occupancy &= !king_bb;

        let mut king_moves = self.king_moves[start] & !board.get_combined_bb(P::color());

        while king_moves != 0 {
            let end = king_moves.pop_lsb();

            if !self.is_sq_under_attack::<P>(end, board, occupancy) {
                move_list.add_move(start, end);
            }
        }
    }
    #[inline(always)]
    fn add_castling_moves<P: PlayerTrait>(&self, move_list: &mut MoveList, board: &Board, occupancy: u64) {
        let file_mask_qs = self.files[Files::B.idx() | Files::C.idx() | Files::D.idx()];
        let file_mask_ks = self.files[Files::F.idx() | Files::G.idx()];

        if P::is_white() {
            if board.can_castle_qs(Color::White)
                && occupancy & self.ranks[Ranks::One as usize] & file_mask_qs == 0
                && !self.is_sq_under_attack::<P>(Square::D1.sq(), board, occupancy)
                && !self.is_sq_under_attack::<P>(Square::C1.sq(), board, occupancy)
            {
                move_list.add_move_with_flags(
                    Square::E1.sq(),
                    Square::C1.sq(),
                    MOVE_TYPE_CASTLE | MOVE_CASTLE_SIDE_QS,
                );
            }
            if board.can_castle_ks(Color::White)
                && occupancy & self.ranks[Ranks::One as usize] & file_mask_ks == 0
                && !self.is_sq_under_attack::<P>(Square::F1.sq(), board, occupancy)
                && !self.is_sq_under_attack::<P>(Square::G1.sq(), board, occupancy)
            {
                move_list.add_move_with_flags(
                    Square::E1.sq(),
                    Square::G1.sq(),
                    MOVE_TYPE_CASTLE | MOVE_CASTLE_SIDE_KS,
                );
            }
        } else {
            if board.can_castle_qs(Color::Black)
                && occupancy & self.ranks[Ranks::Eight as usize] & file_mask_qs == 0
                && !self.is_sq_under_attack::<P>(Square::D8.sq(), board, occupancy)
                && !self.is_sq_under_attack::<P>(Square::C8.sq(), board, occupancy)
            {
                move_list.add_move_with_flags(
                    Square::E8.sq(),
                    Square::C8.sq(),
                    MOVE_TYPE_CASTLE | MOVE_CASTLE_SIDE_QS,
                );
            }
            if board.can_castle_ks(Color::Black)
                && occupancy & self.ranks[Ranks::Eight as usize] & file_mask_ks == 0
                && !self.is_sq_under_attack::<P>(Square::F8.sq(), board, occupancy)
                && !self.is_sq_under_attack::<P>(Square::G8.sq(), board, occupancy)
            {
                move_list.add_move_with_flags(
                    Square::E8.sq(),
                    Square::G8.sq(),
                    MOVE_TYPE_CASTLE | MOVE_CASTLE_SIDE_KS,
                );
            }
        }
    }

    /* -------------------------------------------------------------------------- */
    /*                                Pinned Pieces                               */
    /* -------------------------------------------------------------------------- */
    fn add_pinned_pawn_pushes<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        occupancy: u64,
        pinned_pos: usize,
        mask: u64,
    ) {
        let pawns_bb = 1u64 << pinned_pos;
        let back_rank = P::opposite_back_rank();

        // Single square moves
        let mut pawn_single_moves = if P::is_white() {
            pawns_bb >> 8
        } else {
            pawns_bb << 8
        } & !occupancy;

        // Promotion moves
        let promotion_moves = pawn_single_moves & self.ranks[back_rank as usize] & mask;
        if promotion_moves != 0 {
            move_list.add_promotion(pinned_pos, promotion_moves.lsb_idx());
        } else {
            let non_promotion_moves = pawn_single_moves & self.not_ranks[back_rank as usize] & mask;
            if non_promotion_moves != 0 {
                move_list.add_move(pinned_pos, non_promotion_moves.lsb_idx());
            }

            // Double square moves
            pawn_single_moves &= self.ranks[P::en_passant_rank() as usize];
            let pawn_double_moves = if P::is_white() {
                pawn_single_moves >> 8
            } else {
                pawn_single_moves << 8
            } & mask
                & !occupancy;
            if pawn_double_moves != 0 {
                move_list.add_move(pinned_pos, pawn_double_moves.lsb_idx());
            }
        }
    }
    fn add_pinned_pawn_captures<P: PlayerTrait, C: CaptureSideTrait>(
        &self,
        move_list: &mut MoveList,
        board: &mut Board,
        occupancy: u64,
        pinned_pos: usize,
        mask: u64,
        king_pos: usize,
    ) {
        let enemy_bb = board.get_combined_bb(P::enemy());
        let pawns_bb = 1u64 << pinned_pos;

        let back_rank = P::opposite_back_rank();
        let excluded_file = if C::is_left() { Files::A } else { Files::H };
        let file_mask = self.not_files[excluded_file as usize];
        let captures = if P::is_white() {
            (pawns_bb & file_mask) >> (if C::is_left() { 9 } else { 7 })
        } else {
            (pawns_bb & file_mask) << (if C::is_left() { 7 } else { 9 })
        } & mask;

        // Promotion captures
        let promotion_caps = captures & self.ranks[back_rank as usize] & enemy_bb;
        if promotion_caps != 0 {
            move_list.add_promotion(pinned_pos, promotion_caps.lsb_idx());
        } else {
            if board.en_passant.is_some() {
                let en_passant = board.en_passant.unwrap().sq();
                let en_passant_end =
                    (en_passant as i16 + P::forward_offset()) as usize;
                if captures.is_bit_set(en_passant)
                    && self.validate_en_passant::<P>(
                        board,
                        king_pos,
                        pinned_pos,
                        en_passant_end,
                        occupancy,
                    )
                {
                    move_list.add_move_with_flags(pinned_pos, en_passant_end, MOVE_TYPE_EN_PASSANT);
                }
            }

            let non_promotion_caps = captures & self.not_ranks[back_rank as usize] & enemy_bb;
            if non_promotion_caps != 0 {
                move_list.add_move(pinned_pos, non_promotion_caps.lsb_idx());
            }
        }
    }
    fn add_pinned_moves<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &mut Board,
        occupancy: u64,
        legal_captures: u64,
        blockers: u64,
        king_pos: usize,
        pinned_pos: usize,
        attacker_pos: usize,
    ) {
        let moves_mask = legal_captures | blockers;
        let pin_move_mask = self.slider_range[attacker_pos][king_pos]
            & !board.get_combined_bb(P::color())
            | (1u64 << attacker_pos);

        debug_assert!(board.pieces[pinned_pos].is_some());

        let piece = board.pieces[pinned_pos].unwrap();

        if piece.is_pawn() {
            let king_sq = Square::from_usize(king_pos);
            let pinned_sq = Square::from_usize(pinned_pos);

            // pawns cannot move along a rank
            if king_sq.rank() != pinned_sq.rank() {
                // pinned down a file, so only 1 square or 2 squares forward
                if king_sq.file() == pinned_sq.file() {
                    self.add_pinned_pawn_pushes::<P>(
                        move_list,
                        occupancy,
                        pinned_pos,
                        pin_move_mask & blockers,
                    );
                }
                // diagonal
                else {
                    debug_assert!(
                        Board::distance(king_sq.rank(), pinned_sq.rank())
                            == Board::distance(king_sq.file(), pinned_sq.file())
                    );
                    self.add_pinned_pawn_captures::<P, LeftCapture>(
                        move_list,
                        board,
                        occupancy,
                        pinned_pos,
                        legal_captures & pin_move_mask,
                        king_pos,
                    );
                    self.add_pinned_pawn_captures::<P, RightCapture>(
                        move_list,
                        board,
                        occupancy,
                        pinned_pos,
                        legal_captures & pin_move_mask,
                        king_pos,
                    );
                }
            }
        } else {
            if piece.is_bishop() || piece.is_queen() {
                let mut moves =
                    self.magic_bishop_moves(pinned_pos, occupancy) & moves_mask & pin_move_mask;

                while moves != 0 {
                    move_list.add_move(pinned_pos, moves.pop_lsb());
                }
            }
            
            if piece.is_rook() || piece.is_queen() {
                let mut moves =
                    self.magic_rook_moves(pinned_pos, occupancy) & moves_mask & pin_move_mask;

                while moves != 0 {
                    move_list.add_move(pinned_pos, moves.pop_lsb());
                }
            }
        }
        // king cannot be pinned, knight cannot move if pinned
    }
    fn gen_pin_attackers<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &mut Board,
        occupancy: u64,
        king_pos: usize,
        checkers: u64,
        legal_captures: u64,
        blockers: u64,
        is_bishop: bool,
    ) -> u64 {
        let enemy_color = P::enemy();
        let piece_mask = board.get_bb(Pieces::queen(enemy_color))
            | board.get_bb(if is_bishop {
                Pieces::bishop(enemy_color)
            } else {
                Pieces::rook(enemy_color)
            });
        let mut attackers = if is_bishop {
            self.magic_bishop_moves(king_pos, board.get_combined_bb(P::enemy()))
        } else {
            self.magic_rook_moves(king_pos, board.get_combined_bb(P::enemy()))
        } & piece_mask
            & !checkers;
        let mut pinned_pieces = 0;

        while attackers != 0 {
            let attacker_pos = attackers.pop_lsb();
            let mut occupied =
                self.slider_range[attacker_pos][king_pos] & board.get_combined_bb(P::color());
            if occupied == 0 { continue; }

            let pinned_pos = occupied.pop_lsb();

            // only one piece blocking therefore there is a pin
            if occupied == 0 {
                self.add_pinned_moves::<P>(
                    move_list,
                    board,
                    occupancy,
                    legal_captures,
                    blockers,
                    king_pos,
                    pinned_pos,
                    attacker_pos,
                );
                pinned_pieces.set_bit(pinned_pos);
            }
        }

        pinned_pieces
    }
    fn gen_pinned_pieces<P: PlayerTrait>(
        &self,
        move_list: &mut MoveList,
        board: &mut Board,
        occupancy: u64,
        king_pos: usize,
        checkers: u64,
        legal_captures: u64,
        blockers: u64,
    ) -> u64 {
        self.gen_pin_attackers::<P>(
            move_list,
            board,
            occupancy,
            king_pos,
            checkers,
            legal_captures,
            blockers,
            true,
        ) | self.gen_pin_attackers::<P>(
            move_list,
            board,
            occupancy,
            king_pos,
            checkers,
            legal_captures,
            blockers,
            false,
        )
    }

    fn gen_moves_for_player<P: PlayerTrait>(&self, board: &mut Board, move_list: &mut MoveList) {
        move_list.clear();

        let occupancy = board.get_occupancy();
        let king_pos = board.get_bb(Pieces::king(P::color())).lsb_idx();

        // always generate king moves first
        self.add_king_moves::<P>(move_list, board, occupancy);

        // calculate pieces giving check
        let attacking_king = self.find_enemy_attackers::<P>(king_pos, board, occupancy);

        match attacking_king.count_1s() {
            // double check
            2 => {
                // king moves are the only option, already calculated
            }
            // single check
            1 => {
                let attacker_pos = attacking_king.lsb_idx();

                debug_assert!(board.pieces[attacker_pos].is_some());

                let blockers = if board.pieces[attacker_pos].unwrap().is_knight() {
                    // a knight cannot move out of a pin
                    0
                } else {
                    self.slider_range[king_pos][attacker_pos]
                };

                let move_mask = attacking_king | blockers;
                let pinned = self.gen_pinned_pieces::<P>(
                    move_list,
                    board,
                    occupancy,
                    king_pos,
                    attacking_king,
                    attacking_king,
                    blockers,
                );

                self.add_pawn_moves::<P>(
                    move_list,
                    board,
                    occupancy,
                    pinned,
                    attacking_king,
                    blockers,
                    king_pos,
                );
                self.add_knight_moves::<P>(move_list, board, pinned, move_mask);
                self.add_bishop_moves::<P>(move_list, board, occupancy, pinned, move_mask);
                self.add_rook_moves::<P>(move_list, board, occupancy, pinned, move_mask);
                self.add_queen_moves::<P>(move_list, board, occupancy, pinned, move_mask);
            }
            // not in check - standard move generation
            0 => {
                let pinned = self
                    .gen_pinned_pieces::<P>(move_list, board, occupancy, king_pos, 0, FULL_BB, FULL_BB);

                self.add_castling_moves::<P>(move_list, board, occupancy);
                self.add_pawn_moves::<P>(
                    move_list, board, occupancy, pinned, FULL_BB, FULL_BB, king_pos,
                );
                self.add_knight_moves::<P>(move_list, board, pinned, FULL_BB);
                self.add_bishop_moves::<P>(move_list, board, occupancy, pinned, FULL_BB);
                self.add_rook_moves::<P>(move_list, board, occupancy, pinned, FULL_BB);
                self.add_queen_moves::<P>(move_list, board, occupancy, pinned, FULL_BB);
            }
            _ => {
                panic!("Invalid number of attackers on the king");
            }
        }
    }

    pub fn gen_moves(&self, board: &mut Board, move_list: &mut MoveList) {
        if board.friendly_color().is_white() {
            self.gen_moves_for_player::<WhitePlayer>(board, move_list)
        } else {
            self.gen_moves_for_player::<BlackPlayer>(board, move_list)
        }
    }

    pub fn is_in_check(&self, board: &mut Board) -> bool {
        let occupancy = board.get_occupancy();

        if board.friendly_color().is_white() {
            let king_pos = board.get_bb(Pieces::king(Color::White)).lsb_idx();
            self.find_enemy_attackers::<WhitePlayer>(king_pos, board, occupancy) != 0
        } else {
            let king_pos = board.get_bb(Pieces::king(Color::Black)).lsb_idx();
            self.find_enemy_attackers::<BlackPlayer>(king_pos, board, occupancy) != 0
        }
    }
}

/* -------------------------------------------------------------------------- */
/*                            Rook & Bishop Magics                            */
/* -------------------------------------------------------------------------- */
const ROOK_MAGICS: [u64; 64] = [
    72075735983988992u64,
    162164771226042368u64,
    2774234964794286080u64,
    9295447227374240800u64,
    7133704077631881220u64,
    5404321769049293056u64,
    13871089051341160576u64,
    4647732546161868928u64,
    1154188151204364296u64,
    281623304421378u64,
    9585349132126560768u64,
    324399945019818112u64,
    1266654575591552u64,
    294422971669283848u64,
    9228016932324638976u64,
    422213622698112u64,
    18019346383143456u64,
    13519870926790656u64,
    6917743432679031040u64,
    4611968593184169992u64,
    12170978542791720968u64,
    144159173373870084u64,
    73228578216739328u64,
    2199036100765u64,
    56330731617533952u64,
    148619063654883328u64,
    4625232012420055168u64,
    14988261623278407680u64,
    1478588125675784194u64,
    577024260602875912u64,
    2468254118020653568u64,
    144256209032118404u64,
    40577751509369480u64,
    6917564213158219778u64,
    9007478444400656u64,
    20839044434890752u64,
    4611976300242928640u64,
    4617878489423415312u64,
    11278859869620225u64,
    288230653210657060u64,
    576531123197214720u64,
    844699816624161u64,
    4616198431329755136u64,
    1513221569692893216u64,
    12125942013883416584u64,
    4613005570896036100u64,
    72066394459734032u64,
    1765429764459462660u64,
    342291713626218624u64,
    22518273021051200u64,
    9464597434109056u64,
    613052534176650752u64,
    20547690614100224u64,
    140746078552192u64,
    45044801233552384u64,
    27028749086179840u64,
    290556685111457u64,
    288865903000617090u64,
    1161084417409045u64,
    289075918041778209u64,
    2522578810537804930u64,
    1298444514277720065u64,
    1143496522109444u64,
    2305843716071555138u64,
];
const BISHOP_MAGICS: [u64; 64] = [
    1179020146311185u64,
    145267478427205635u64,
    4504158111531524u64,
    9224516499644878888u64,
    144680405855912002u64,
    4619005622497574912u64,
    1130315234418688u64,
    5349125176573952u64,
    6071010655858065920u64,
    20310248111767713u64,
    1297094009090539520u64,
    4616233778910625860u64,
    2305849615159678976u64,
    74381998193642242u64,
    1407684255942661u64,
    2305862803678299144u64,
    22535635693734016u64,
    4503608284938884u64,
    11259016393073153u64,
    108650578499878976u64,
    41095363813851170u64,
    9232520132522148096u64,
    70385943187776u64,
    9227035893351617024u64,
    1155182103739172867u64,
    11530343153862181120u64,
    2295791083930624u64,
    1130297991168512u64,
    281543712980996u64,
    307513611096490433u64,
    2289183226103316u64,
    4612816874811392128u64,
    4547891544985604u64,
    3458958372559659520u64,
    303473866573824u64,
    1729558217427519744u64,
    5633914760597520u64,
    1441434463836899328u64,
    20269028707403544u64,
    149744981853258752u64,
    2252933819802113u64,
    1163074498090533888u64,
    4681729134575680u64,
    4621485970984798208u64,
    367078571518203970u64,
    72621098075685120u64,
    1225544256278495744u64,
    1411779381045761u64,
    5333500077688291352u64,
    4716913491968128u64,
    148627764202701056u64,
    1688850967695425u64,
    17781710002178u64,
    9243644149415084036u64,
    218426849703891488u64,
    9009415596316677u64,
    1412882374067224u64,
    279186509824u64,
    20407489916899328u64,
    4614113755159331840u64,
    144119586390940160u64,
    11547234118442230016u64,
    5188151323463779840u64,
    435758450535334272u64,
];
