use std::fmt::{Display, Formatter, Result};

use crate::engine::piece::{Color, Pieces};
use crate::engine::r#move::{Move, MoveUtils, UndoInfo};
use crate::engine::square::Square;

use super::bitboard::BitBoardUtils;

pub const BLACK_CASTLE_QS: u8 = 0b0001;
pub const BLACK_CASTLE_KS: u8 = 0b0010;
pub const WHITE_CASTLE_QS: u8 = 0b0100;
pub const WHITE_CASTLE_KS: u8 = 0b1000;

pub const WHITE_CASTLE: u8 = WHITE_CASTLE_KS | WHITE_CASTLE_QS;
pub const BLACK_CASTLE: u8 = BLACK_CASTLE_KS | BLACK_CASTLE_QS;

pub const STARTING_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(PartialEq, Clone, Copy)]
pub struct Board {
    current_color: Color,

    fifty_move: usize,
    full_move_count: usize,

    castling: u8,
    pub en_passant: Option<Square>,

    pub pieces: [Option<Pieces>; 64],

    piece_bitboards: [u64; 12],
    combined_bitboards: [u64; 2],

    zobrist_table: [[u64; 12]; 64],

    zobrist_hash: u64,
}

impl Board {
    fn zero_boards(&mut self) {
        self.piece_bitboards.fill(0);
        self.combined_bitboards.fill(0);
        self.pieces.fill(None);
    }
    fn load_fen(&mut self, fen: &str) -> std::result::Result<(), String> {
        self.zero_boards();

        let args: Vec<&str> = fen.split_whitespace().collect();
        if args.len() != 6 {
            return Err(String::from("Expected 6 whitespace delimited arguments"));
        }

        // parse board
        let mut pos = 0;
        let mut square: usize = 0;
        let fen_board_arg = args[0];
        while square < 64 && pos < fen_board_arg.len() {
            let mut piece = None;

            match fen_board_arg.chars().nth(pos).unwrap() {
                'p' => piece = Some(Pieces::BlackPawn),
                'n' => piece = Some(Pieces::BlackKnight),
                'b' => piece = Some(Pieces::BlackBishop),
                'r' => piece = Some(Pieces::BlackRook),
                'q' => piece = Some(Pieces::BlackQueen),
                'k' => piece = Some(Pieces::BlackKing),

                'P' => piece = Some(Pieces::WhitePawn),
                'N' => piece = Some(Pieces::WhiteKnight),
                'B' => piece = Some(Pieces::WhiteBishop),
                'R' => piece = Some(Pieces::WhiteRook),
                'Q' => piece = Some(Pieces::WhiteQueen),
                'K' => piece = Some(Pieces::WhiteKing),

                '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' => {
                    square += fen_board_arg
                        .chars()
                        .nth(pos)
                        .unwrap()
                        .to_string()
                        .parse::<usize>()
                        .unwrap()
                }

                '/' | ' ' => {}
                _ => return Err(String::from("Unrecognised character in FEN")),
            }

            if let Some(my_piece) = piece {
                self.get_bb_mut(my_piece).set_bit(square);
                self.pieces[square] = piece;
                square += 1;
            }

            pos += 1;
        }

        if square < 64 {
            return Err(String::from("Expected 64 squares in FEN"));
        }

        // parse current player
        let player_arg = args[1];
        self.current_color = match player_arg.chars().next() {
            Some(c) => match c {
                'w' => Color::White,
                'b' => Color::Black,
                _ => return Err(String::from("Expected w/b for current player")),
            },
            None => return Err(String::from("Expected w/b for current player")),
        };

        // parse castling rights
        self.castling = 0;
        for c in args[2].chars() {
            match c {
                'q' => self.castling |= BLACK_CASTLE_QS,
                'k' => self.castling |= BLACK_CASTLE_KS,
                'Q' => self.castling |= WHITE_CASTLE_QS,
                'K' => self.castling |= WHITE_CASTLE_KS,
                '-' => break,
                _ => return Err(String::from("Invalid character in castling rights")),
            }
        }

        // parse en passant
        let en_passant_arg = args[3];
        if en_passant_arg == "-" {
            self.en_passant = None;
        } else {
            self.en_passant = Square::from_notation(en_passant_arg);
        }

        // parse fifty_move
        self.fifty_move = args[4].parse().unwrap();

        // parse fullmove count
        self.full_move_count = args[5].parse().unwrap();

        *self.get_combined_bb_mut(Color::White) = self.get_bb(Pieces::WhitePawn)
            | self.get_bb(Pieces::WhiteKnight)
            | self.get_bb(Pieces::WhiteBishop)
            | self.get_bb(Pieces::WhiteRook)
            | self.get_bb(Pieces::WhiteQueen)
            | self.get_bb(Pieces::WhiteKing);

        *self.get_combined_bb_mut(Color::Black) = self.get_bb(Pieces::BlackPawn)
            | self.get_bb(Pieces::BlackKnight)
            | self.get_bb(Pieces::BlackBishop)
            | self.get_bb(Pieces::BlackRook)
            | self.get_bb(Pieces::BlackQueen)
            | self.get_bb(Pieces::BlackKing);

        Ok(())
    }

    pub fn hash(&self) -> u64 {
        self.zobrist_hash
    }

    pub fn rand_zobrist_table(&mut self, rng: &mut impl rand::Rng) {
        for sq in 0..64 {
            for piece in 0..12 {
                self.zobrist_table[sq][piece] = rng.gen();
            }
        }
    }

    pub fn make_move(&mut self, my_move: Move, info: &mut UndoInfo) {
        // load data from move
        let start = my_move.get_move_start() as usize;
        let end = my_move.get_move_end() as usize;
        let piece = my_move.get_move_piece();

        // store current colors
        let friendly_color = self.friendly_color();
        let enemy_color = self.enemy_color();

        // save current state
        info.castling = self.castling;
        info.fifty_move = self.fifty_move;
        info.en_passant = self.en_passant;
        info.captured = self.pieces[end];

        // store start and end pieces
        let start_piece = self.pieces[start];
        let end_piece = self.pieces[end];

        // check move
        debug_assert!(start != end);
        debug_assert!(Square::valid_sq(start as i16));
        debug_assert!(Square::valid_sq(end as i16));
        debug_assert!(start_piece.is_some());
        let start_piece = start_piece.unwrap();

        debug_assert!(start_piece.color() == self.friendly_color());
        debug_assert!(end_piece.is_none() || start_piece.color() != end_piece.unwrap().color());

        // remove start piece from start square
        self.zobrist_hash ^= self.zobrist_table[start][start_piece.idx()];

        // update fifty_move
        if self.pieces[start].unwrap().is_pawn() || self.pieces[end].is_some() {
            self.fifty_move = 0;
        } else {
            self.fifty_move += 1;
        }

        // process move
        match my_move.get_move_type() {
            super::r#move::MOVE_TYPE_EN_PASSANT => {
                debug_assert!(self.en_passant.is_some());

                let friendly_pawn = Pieces::pawn(friendly_color);
                let enemy_pawn = Pieces::pawn(enemy_color);
                let en_passant_sq = self.en_passant.unwrap().sq();
                
                // add start piece to en_passant square
                self.zobrist_hash ^= self.zobrist_table[en_passant_sq][friendly_pawn.idx()];
                // remove end piece from end square
                self.zobrist_hash ^= self.zobrist_table[end][enemy_pawn.idx()];

                // friendly piece bb
                self.get_bb_mut(friendly_pawn)
                    .clear_bit(start)
                    .set_bit(en_passant_sq);

                // enemy piece bb
                self.get_bb_mut(enemy_pawn).clear_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .clear_bit(start)
                    .set_bit(en_passant_sq);

                // enemy combined bb
                self.get_combined_bb_mut(enemy_color).clear_bit(end);

                // piece array
                self.pieces[en_passant_sq] = Some(friendly_pawn);
                self.pieces[start] = None;
                self.pieces[end] = None;

                self.en_passant = None;
            }
            super::r#move::MOVE_TYPE_CASTLE => {
                let friendly_king = Pieces::king(friendly_color);
                let friendly_rook = Pieces::rook(friendly_color);

                // add king to end square
                self.zobrist_hash ^= self.zobrist_table[end][friendly_king.idx()];
                
                debug_assert_eq!(start_piece, friendly_king);

                // friendly king bb
                self.get_bb_mut(friendly_king).clear_bit(start).set_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .clear_bit(start)
                    .set_bit(end);

                let offset = start & 0b111000;

                match piece {
                    // queenside
                    super::r#move::MOVE_CASTLE_SIDE_QS => {
                        debug_assert!(self.can_castle_qs(friendly_color));
                        debug_assert!(self.pieces[offset].is_some());
                        debug_assert_eq!(self.pieces[offset].unwrap(), friendly_rook);

                        // remove rook from start square
                        self.zobrist_hash ^= self.zobrist_table[offset][friendly_rook.idx()];
                        // add rook to end square
                        self.zobrist_hash ^= self.zobrist_table[offset + 3][friendly_rook.idx()];

                        // friendly rook bb
                        self.get_bb_mut(friendly_rook)
                            .clear_bit(offset)
                            .set_bit(offset + 3);

                        // friendly combined bb
                        self.get_combined_bb_mut(friendly_color)
                            .clear_bit(offset)
                            .set_bit(offset + 3);

                        // pieces array
                        self.pieces[offset + 3] = Some(friendly_rook);
                        self.pieces[offset] = None;
                    }
                    // kingside
                    _ => {
                        debug_assert!(self.can_castle_ks(friendly_color));
                        debug_assert!(self.pieces[offset + 7].is_some());
                        debug_assert_eq!(self.pieces[offset + 7].unwrap(), friendly_rook);

                        // remove rook from start square
                        self.zobrist_hash ^= self.zobrist_table[offset + 7][friendly_rook.idx()];
                        // add rook to end square
                        self.zobrist_hash ^= self.zobrist_table[offset + 5][friendly_rook.idx()];

                        // friendly rook bb
                        self.get_bb_mut(friendly_rook)
                            .clear_bit(offset + 7)
                            .set_bit(offset + 5);

                        // friendly combined bb
                        self.get_combined_bb_mut(friendly_color)
                            .clear_bit(offset + 7)
                            .set_bit(offset + 5);

                        // pieces array
                        self.pieces[offset + 5] = Some(friendly_rook);
                        self.pieces[offset + 7] = None;
                    }
                };

                // the active side can no longer castle
                self.disable_castle_for_color(friendly_color);

                // piece array
                self.pieces[end] = Some(friendly_king);
                self.pieces[start] = None;

                self.en_passant = None;
            }
            super::r#move::MOVE_TYPE_PROMOTION => {
                let friendly_pawn = Pieces::pawn(friendly_color);

                debug_assert_eq!(start_piece, friendly_pawn);

                // decode the promotion piece
                let promotion_piece = match piece {
                    super::r#move::MOVE_PROMOTION_PIECE_KNIGHT => Pieces::knight(friendly_color),
                    super::r#move::MOVE_PROMOTION_PIECE_BISHOP => Pieces::bishop(friendly_color),
                    super::r#move::MOVE_PROMOTION_PIECE_ROOK => Pieces::rook(friendly_color),
                    super::r#move::MOVE_PROMOTION_PIECE_QUEEN => Pieces::queen(friendly_color),
                    _ => panic!("Couldn't match the promotion piece"),
                };

                // clear the end piece if this is a capture
                if let Some(end_piece) = end_piece {
                    // remove enemy piece from end square
                    self.zobrist_hash ^= self.zobrist_table[end][end_piece.idx()];

                    // enemy piece bb
                    self.get_bb_mut(end_piece).clear_bit(end);

                    // enemy combined bb
                    self.get_combined_bb_mut(enemy_color).clear_bit(end);

                    // additionally, if the captured piece was a rook, update the castling state
                    if end_piece.is_rook() {
                        self.disable_castle_from_sq(end);
                    }
                }
                
                // add promotion piece to end square
                self.zobrist_hash ^= self.zobrist_table[end][promotion_piece.idx()];

                // friendly piece bb for pawn and promotion piece
                self.get_bb_mut(friendly_pawn).clear_bit(start);
                self.get_bb_mut(promotion_piece).set_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .clear_bit(start)
                    .set_bit(end);

                // pieces array
                self.pieces[start] = None;
                self.pieces[end] = Some(promotion_piece);
                self.en_passant = None;
            }
            _ => {
                self.en_passant = if start_piece.is_pawn() && Board::distance(end, start) == 16 {
                    Some(Square::from_usize((end + start) / 2))
                } else {
                    None
                };

                if let Some(end_piece) = end_piece {
                    // remove enemy piece from end square
                    self.zobrist_hash ^= self.zobrist_table[end][end_piece.idx()];

                    // enemy piece bb
                    self.get_bb_mut(end_piece).clear_bit(end);

                    // enemy combined bb
                    self.get_combined_bb_mut(enemy_color).clear_bit(end);

                    // if friendly piece takes enemy rook, then that enemy side cannot be castled to
                    if end_piece.is_rook() {
                        self.disable_castle_from_sq(end);
                    }
                }

                // if the king moves then that player cannot castle
                if start_piece.is_king() {
                    self.disable_castle_for_color(friendly_color);
                }
                // if friendly rook moves, then that side cannot be castled to
                else if start_piece.is_rook() {
                    self.disable_castle_from_sq(start);
                }

                // add friendly piece to end square
                self.zobrist_hash ^= self.zobrist_table[end][start_piece.idx()];

                // friendly piece bb
                self.get_bb_mut(start_piece).clear_bit(start).set_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .clear_bit(start)
                    .set_bit(end);

                // pieces array
                self.pieces[end] = self.pieces[start];
                self.pieces[start] = None;
            }
        }

        self.current_color = self.current_color.enemy();
    }
    pub fn undo_move(&mut self, my_move: Move, info: &UndoInfo) {
        // load data from move
        let start = my_move.get_move_start() as usize;
        let end = my_move.get_move_end() as usize;
        let piece = my_move.get_move_piece();

        // store current colors
        let friendly_color = self.enemy_color();
        let enemy_color = self.friendly_color();

        // load previous state
        self.current_color = self.current_color.enemy();
        self.castling = info.castling;
        self.fifty_move = info.fifty_move;
        self.en_passant = info.en_passant;
        let captured_piece = info.captured;

        // store end piece
        let end_piece = self.pieces[end];

        // check move
        debug_assert!(start != end);
        debug_assert!(Square::valid_sq(start as i16));
        debug_assert!(Square::valid_sq(end as i16));
        debug_assert!(self.pieces[start].is_none());

        if my_move.get_move_type() != super::r#move::MOVE_TYPE_EN_PASSANT {
            debug_assert!(self.pieces[end].is_some());
            debug_assert!(self.pieces[end].unwrap().color() == friendly_color);
        }

        // process move
        match my_move.get_move_type() {
            super::r#move::MOVE_TYPE_EN_PASSANT => {
                let friendly_pawn = Pieces::pawn(friendly_color);
                let enemy_pawn = Pieces::pawn(enemy_color);
                let en_passant_sq = self.en_passant.unwrap().sq();

                // add start piece to start square
                self.zobrist_hash ^= self.zobrist_table[start][friendly_pawn.idx()];
                // remove start piece from en_passant square
                self.zobrist_hash ^= self.zobrist_table[en_passant_sq][friendly_pawn.idx()];
                // add end piece to end square
                self.zobrist_hash ^= self.zobrist_table[end][enemy_pawn.idx()];

                debug_assert!(self.pieces[end].is_none());
                debug_assert!(self.en_passant.is_some());
                debug_assert!(self.pieces[self.en_passant.unwrap().sq()] == Some(friendly_pawn));

                // friendly piece bb
                self.get_bb_mut(friendly_pawn)
                    .set_bit(start)
                    .clear_bit(en_passant_sq);

                // enemy piece bb
                self.get_bb_mut(enemy_pawn).set_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .set_bit(start)
                    .clear_bit(en_passant_sq);

                // enemy combined bb
                self.get_combined_bb_mut(enemy_color).set_bit(end);

                // piece array
                self.pieces[en_passant_sq] = None;
                self.pieces[start] = Some(friendly_pawn);
                self.pieces[end] = Some(enemy_pawn);
            }
            super::r#move::MOVE_TYPE_CASTLE => {
                let friendly_king = Pieces::king(friendly_color);
                let friendly_rook = Pieces::rook(friendly_color);

                debug_assert_eq!(end_piece.unwrap(), friendly_king);

                // add friendly king to start square
                self.zobrist_hash ^= self.zobrist_table[start][friendly_king.idx()];
                // remove friendly king from end square
                self.zobrist_hash ^= self.zobrist_table[end][friendly_king.idx()];

                // friendly piece bb
                self.get_bb_mut(friendly_king).set_bit(start).clear_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .set_bit(start)
                    .clear_bit(end);

                // pieces array
                self.pieces[start] = Some(friendly_king);
                self.pieces[end] = None;

                let offset = start & 0b111000;
                match piece {
                    // queenside
                    super::r#move::MOVE_CASTLE_SIDE_QS => {
                        // add rook to start square
                        self.zobrist_hash ^= self.zobrist_table[offset][friendly_rook.idx()];
                        // remove rook from end square
                        self.zobrist_hash ^= self.zobrist_table[offset + 3][friendly_rook.idx()];

                        // friendly rook bb
                        self.get_bb_mut(friendly_rook)
                            .set_bit(offset)
                            .clear_bit(offset + 3);

                        // friendly combined bb
                        self.get_combined_bb_mut(friendly_color)
                            .set_bit(offset)
                            .clear_bit(offset + 3);

                        // pieces array
                        self.pieces[offset + 3] = None;
                        self.pieces[offset] = Some(friendly_rook);
                    }
                    // kingside
                    _ => {
                        // add rook to start square
                        self.zobrist_hash ^= self.zobrist_table[offset + 7][friendly_rook.idx()];
                        // remove rook from end square
                        self.zobrist_hash ^= self.zobrist_table[offset + 5][friendly_rook.idx()];

                        // friendly rook bb
                        self.get_bb_mut(friendly_rook)
                            .set_bit(offset + 7)
                            .clear_bit(offset + 5);

                        // friendly combined bb
                        self.get_combined_bb_mut(friendly_color)
                            .set_bit(offset + 7)
                            .clear_bit(offset + 5);

                        // pieces array
                        self.pieces[offset + 5] = None;
                        self.pieces[offset + 7] = Some(friendly_rook);
                    }
                };
            }
            super::r#move::MOVE_TYPE_PROMOTION => {
                let friendly_pawn = Pieces::pawn(friendly_color);
                let promotion_piece = self.pieces[end].unwrap();
                
                // add friendly pawn to start square
                self.zobrist_hash ^= self.zobrist_table[start][friendly_pawn.idx()];
                // remove promotion piece from end square
                self.zobrist_hash ^= self.zobrist_table[end][promotion_piece.idx()];

                // friendly piece bb
                self.get_bb_mut(friendly_pawn).set_bit(start);
                self.get_bb_mut(promotion_piece).clear_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .set_bit(start)
                    .clear_bit(end);

                // piece array
                self.pieces[start] = Some(friendly_pawn);
                self.pieces[end] = captured_piece;

                if let Some(captured_piece) = captured_piece {
                    // remove captured piece from end square
                    self.zobrist_hash ^= self.zobrist_table[end][captured_piece.idx()];
                    
                    // enemy piece bb
                    self.get_bb_mut(captured_piece).set_bit(end);

                    // enemy combined bb
                    self.get_combined_bb_mut(enemy_color).set_bit(end);
                }
            }
            _ => {
                let end_piece = end_piece.unwrap();

                // add friendly piece to start square
                self.zobrist_hash ^= self.zobrist_table[start][end_piece.idx()];
                // remove friendly piece from end square
                self.zobrist_hash ^= self.zobrist_table[end][end_piece.idx()];

                // friendly piece bb
                self.get_bb_mut(end_piece)
                    .set_bit(start)
                    .clear_bit(end);

                // friendly combined bb
                self.get_combined_bb_mut(friendly_color)
                    .set_bit(start)
                    .clear_bit(end);

                // piece array
                self.pieces[start] = self.pieces[end];
                self.pieces[end] = captured_piece;

                if let Some(captured_piece) = captured_piece {
                    // remove captured piece from end square
                    self.zobrist_hash ^= self.zobrist_table[end][captured_piece.idx()];

                    // enemy piece bb
                    self.get_bb_mut(captured_piece).set_bit(end);

                    // enemy combined bb
                    self.get_combined_bb_mut(enemy_color).set_bit(end);
                }
            }
        }
    }

    #[inline(always)]
    fn disable_castle_for_color(&mut self, color: Color) {
        if color.is_white() {
            self.castling &= !WHITE_CASTLE;
        } else {
            self.castling &= !BLACK_CASTLE;
        }
    }
    #[inline(always)]
    fn disable_castle_from_sq(&mut self, sq: usize) {
        match Square::from_usize(sq) {
            Square::A1 => self.castling &= !WHITE_CASTLE_QS,
            Square::H1 => self.castling &= !WHITE_CASTLE_KS,
            Square::A8 => self.castling &= !BLACK_CASTLE_QS,
            Square::H8 => self.castling &= !BLACK_CASTLE_KS,
            _ => {}
        }
    }

    #[inline(always)]
    pub fn can_castle_qs(&self, color: Color) -> bool {
        if color.is_white() {
            self.castling & WHITE_CASTLE_QS != 0
        } else {
            self.castling & BLACK_CASTLE_QS != 0
        }
    }
    #[inline(always)]
    pub fn can_castle_ks(&self, color: Color) -> bool {
        if color.is_white() {
            self.castling & WHITE_CASTLE_KS != 0
        } else {
            self.castling & BLACK_CASTLE_KS != 0
        }
    }

    #[inline(always)]
    pub fn distance(a: usize, b: usize) -> usize {
        if a > b {
            a - b
        } else {
            b - a
        }
    }

    #[inline(always)]
    pub fn friendly_color(&self) -> Color {
        self.current_color
    }
    #[inline(always)]
    pub fn enemy_color(&self) -> Color {
        self.current_color.enemy()
    }

    #[inline(always)]
    pub fn get_bb(&self, piece: Pieces) -> u64 {
        self.piece_bitboards[piece.idx()]
    }
    #[inline(always)]
    pub fn get_bb_mut(&mut self, piece: Pieces) -> &mut u64 {
        &mut self.piece_bitboards[piece.idx()]
    }
    #[inline(always)]
    pub fn get_combined_bb(&self, color: Color) -> u64 {
        self.combined_bitboards[color.idx()]
    }
    #[inline(always)]
    pub fn get_combined_bb_mut(&mut self, color: Color) -> &mut u64 {
        &mut self.combined_bitboards[color.idx()]
    }
    #[inline(always)]
    pub fn get_occupancy(&self) -> u64 {
        self.get_combined_bb(Color::White) | self.get_combined_bb(Color::Black)
    }

    pub fn to_fen(&self) -> String {
        let mut result = String::new();
        let mut square: usize = 0;
        let mut rank = 0;
        let mut file = 0;
        let mut empty = 0;

        // board
        while rank < 8 {
            debug_assert!(Square::valid_rf(rank, file));
            debug_assert!(Square::valid_sq(square as i16));

            // process piece
            if let Some(piece) = self.pieces[square] {
                if empty != 0 {
                    result.push_str(&empty.to_string());
                }
                empty = 0;

                result.push(piece.notation());
            } else {
                empty += 1;
            }

            square += 1;
            file += 1;

            // reached end of row
            if file > 7 {
                if empty != 0 {
                    result.push_str(&empty.to_string());
                }
                if rank < 7 {
                    result.push('/');
                }
                rank += 1;
                empty = 0;
                file = 0;
            }
        }

        result.push_str(&format!(" {} ", self.friendly_color().as_letter()));

        if self.castling == 0 {
            result.push('-');
        } else {
            if self.castling & WHITE_CASTLE_QS != 0 {
                result.push('Q');
            }
            if self.castling & BLACK_CASTLE_QS != 0 {
                result.push('q');
            }
            if self.castling & WHITE_CASTLE_KS != 0 {
                result.push('K');
            }
            if self.castling & BLACK_CASTLE_KS != 0 {
                result.push('k');
            }
        }

        result.push_str(&format!(
            " {} {} {}",
            if let Some(square) = self.en_passant {
                square.notation()
            } else {
                "-".to_string()
            },
            self.fifty_move,
            self.full_move_count
        ));

        result
    }

    pub fn new(fen: &str) -> std::result::Result<Board, String> {
        let mut board = Board {
            current_color: Color::White,
            fifty_move: 0,
            full_move_count: 0,
            castling: 0b1111,
            en_passant: None,
            pieces: [None; 64],
            piece_bitboards: [0; 12],
            combined_bitboards: [0; 2],
            zobrist_table: [[0; 12]; 64],
            zobrist_hash: 0,
        };

        // init zobrist table
        board.rand_zobrist_table(&mut rand::thread_rng());

        // load fen
        match board.load_fen(fen) {
            Ok(()) => {
                // init zobrist hash
                for sq in 0..64 {
                    if let Some(piece) = board.pieces[sq] {
                        board.zobrist_hash ^= board.zobrist_table[sq][piece.idx()]
                    }
                }
                Ok(board)
            },
            Err(msg) => {
                Err(msg)
            }
        }
    }
}

impl Default for Board {
    fn default() -> Board {
        Board::new(STARTING_FEN).unwrap()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut result = concat!("    a b c d e f g h\n", "  ╭─────────────────╮\n").to_string();

        for rank in 0..8 {
            result.push_str(&format!("{} │ ", 8 - rank));
            for file in 0..8 {
                result.push_str(&format!(
                    "{} ",
                    match self.pieces[Square::from_rf(rank, file) as usize] {
                        Some(piece) => piece.notation(),
                        None => ' ',
                    }
                ));
            }
            result.push_str(&format!("│ {}\n", 8 - rank));
        }

        result.push_str(concat!("  ╰─────────────────╯\n", "    a b c d e f g h\n"));
        result.push_str(if self.friendly_color() == Color::White {
            "     White to move"
        } else {
            "     Black to move"
        });

        write!(f, "{}", result)
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::movegen::{MoveGenerator, MoveList};
    use super::*;

    fn fen_test(fen: &str) -> bool {
        let board = Board::new(fen).unwrap();
        board.to_fen().eq(fen)
    }

    #[test]
    fn fen() {
        assert!(fen_test("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2"));
        assert!(fen_test("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3"));
        assert!(fen_test(
            "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2"
        ));
        assert!(fen_test(
            "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2"
        ));
        assert!(fen_test(
            "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9"
        ));
        assert!(fen_test("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4"));
        assert!(fen_test(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b QK - 3 2"
        ));
        assert!(fen_test("4k3/8/8/5R2/8/8/8/4K3 b - - 0 1"));
        assert!(fen_test("8/4k3/8/8/4R3/8/8/4K3 b - - 0 1"));
        assert!(fen_test("4k3/6N1/5b2/4R3/8/8/8/4K3 b - - 0 1"));
        assert!(fen_test("4k3/8/6n1/4R3/8/8/8/4K3 b - - 0 1"));
        assert!(fen_test("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1"));
        assert!(fen_test("8/8/8/1k6/3Pp3/8/8/4KQ2 b - d3 0 1"));
        assert!(fen_test("4k3/8/4r3/8/8/4Q3/8/2K5 b - - 0 1"));
        assert!(fen_test("8/8/8/8/k2Pp2Q/8/8/2K5 b - d3 0 1"));
    }

    fn undo_test(fen: &str) -> bool {
        let mut board = Board::new(fen).unwrap();

        let mut move_list = MoveList::new();
        let generator = MoveGenerator::new();
        
        generator.gen_moves(&mut board, &mut move_list);

        let mut info = UndoInfo::default();
        for i in 0..move_list.len() {
            let mut test_board = board;

            test_board.make_move(move_list.at(i), &mut info);
            test_board.undo_move(move_list.at(i), &info);

            if test_board != board {
                println!("{}", move_list.at(i).move_to_string());
                return false;
            }
        }

        true
    }

    #[test]
    fn undo() {
        assert!(undo_test(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1R1K b kq - 1 1"
        ));
        assert!(undo_test("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2"));
        assert!(undo_test("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3"));
        assert!(undo_test(
            "r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2"
        ));
        assert!(undo_test(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
        ));
        assert!(undo_test(
            "r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2"
        ));
        assert!(undo_test(
            "rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9"
        ));
        assert!(undo_test("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4"));
        assert!(undo_test(
            "2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b QK - 3 2"
        ));
        assert!(undo_test("4k3/8/8/5R2/8/8/8/4K3 b - - 0 1"));
        assert!(undo_test("8/4k3/8/8/4R3/8/8/4K3 b - - 0 1"));
        assert!(undo_test("4k3/6N1/5b2/4R3/8/8/8/4K3 b - - 0 1"));
        assert!(undo_test("4k3/8/6n1/4R3/8/8/8/4K3 b - - 0 1"));
        assert!(undo_test("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1"));
        assert!(undo_test("8/8/8/1k6/3Pp3/8/8/4KQ2 b - d3 0 1"));
        assert!(undo_test("4k3/8/4r3/8/8/4Q3/8/2K5 b - - 0 1"));
        assert!(undo_test("8/8/8/8/k2Pp2Q/8/8/2K5 b - d3 0 1"));
        assert!(undo_test(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8"
        ));
        assert!(undo_test(
            "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
        ));
        assert!(undo_test(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1"
        ));
        assert!(undo_test(
            "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1"
        ));
        assert!(undo_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1"));
        assert!(undo_test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1"
        ));
        assert!(undo_test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        ));
    }
}
