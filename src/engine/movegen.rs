use crate::engine::piece::Color;
use crate::engine::square::Square;
use crate::engine::bitboard::BitBoardUtils;

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

pub struct MoveGenerator {
    rook_masks: [u64; 64],
    bishop_masks: [u64; 64],

    rook_magic_shifts: [usize; 64],
    bishop_magic_shifts: [usize; 64],

    rook_moves: [[u64; 4096]; 64],
    bishop_moves: [[u64; 4096]; 64],

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
        let rank = start.rank();
        let file = start.file();
        let mut result = 0;

        for r in rank as i16-1..=1 {
            result.set_bit(Square::from_rf(r as usize, file) as usize);
        }

        for r in rank+1..=6 {
            result.set_bit(Square::from_rf(r, file) as usize);
        }

        for f in file as i16-1..=1 {
            result.set_bit(Square::from_rf(rank, f as usize) as usize);
        }

        for f in file+1..=6 {
            result.set_bit(Square::from_rf(rank, f) as usize);
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
            result.set_bit(Square::from_rf(r as usize, f as usize) as usize);
            r -= 1;
            f -= 1;
        }

        r = rank+1;
        f = file-1;
        while r <= 6 && f >= 1 {
            result.set_bit(Square::from_rf(r as usize, f as usize) as usize);
            r += 1;
            f -= 1;
        }

        r = rank-1;
        f = file+1;
        while r >= 1 && f <= 6 {
            result.set_bit(Square::from_rf(r as usize, f as usize) as usize);
            r -= 1;
            f += 1;
        }

        r = rank+1;
        f = file+1;
        while r <= 6 && f <= 6 {
            result.set_bit(Square::from_rf(r as usize, f as usize) as usize);
            r += 1;
            f += 1;
        }

        result
    }

    fn gen_rook_moves(&self, start: Square, occupancy: u64) -> u64 {
        let rank = start.rank() as i16;
        let file = start.file() as i16;
        let mut result = 0;

        for r in rank-1..=0 {
            let pos = Square::from_rf(r as usize, file as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
        }

        for r in rank+1..=7 {
            let pos = Square::from_rf(r as usize, file as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
        }

        for f in file-1..=0 {
            let pos = Square::from_rf(rank as usize, f as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
        }

        for f in file+1..=7 {
            let pos = Square::from_rf(rank as usize, f as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
        }

        result
    }
    fn gen_bishop_moves(&self, start: Square, occupancy: u64) ->  u64 {
        let rank = start.rank() as i16;
        let file = start.file() as i16;
        let mut result = 0;

        let mut r = rank - 1;
        let mut f = file - 1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r -= 1;
            f -= 1;
        }

        r = rank+1;
        f = file-1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r += 1;
            f -= 1;
        }

        r = rank-1;
        f = file+1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize) as usize;
            result.set_bit(pos);
            if occupancy.is_bit_set(pos) {
                break;
            }
            r -= 1;
            f += 1;
        }

        r = rank+1;
        f = file+1;
        while Square::valid_rf(r, f) {
            let pos = Square::from_rf(r as usize, f as usize) as usize;
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
            [-1,  0],
            [-1,  1],
            [ 0, -1],
            [ 0,  1],
            [ 1, -1],
            [ 1,  0],
            [-1,  1],
        ];

        for &vector in &MOVE_VECTORS {
            let f = file + vector[0];
            let r = rank + vector[1];
            if Square::valid_rf(r, f) {
                result.set_bit(Square::from_rf(r as usize, f as usize) as usize);
            }
        }

        result
    }
    fn gen_knight_moves(&self, start: Square) -> u64 {
        let mut result = 0;

        let rank = start.rank() as i16;
        let file = start.file() as i16;

        const MOVE_VECTORS: [[i16; 2]; 8] = [
            [ -1,  2 ],
            [ -2,  1 ],
            [  1,  2 ],
            [  2,  1 ],
            [ -1, -2 ],
            [ -2, -1 ],
            [  1, -2 ],
            [  2, -1 ],
        ];

        for &vector in &MOVE_VECTORS {
            let f = file + vector[0];
            let r = rank + vector[1];
            if Square::valid_rf(r, f) {
                result.set_bit(Square::from_rf(r as usize, f as usize) as usize);
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
            for idx in (1 << self.rook_masks[i].count_1s())-1..=0 {
                let indexed_mask = self.idx_to_u64(idx, self.rook_masks[i]);
                let key = (ROOK_MAGICS[i] * indexed_mask) >> self.rook_magic_shifts[i];
                self.rook_moves[i][key as usize] = self.gen_rook_moves(sq, indexed_mask);
            }
            for idx in (1 << self.bishop_masks[i].count_1s())-1..=0 {
                let indexed_mask = self.idx_to_u64(idx, self.bishop_masks[i]);
                let key = (BISHOP_MAGICS[i] * indexed_mask) >> self.bishop_magic_shifts[i];
                self.bishop_moves[i][key as usize] = self.gen_bishop_moves(sq, indexed_mask);
            }

            // knight moves
            self.knight_moves[i] = self.gen_knight_moves(sq);

            // king moves
            self.king_moves[i] = self.gen_knight_moves(sq);

            // pawn attacks
            if sq.rank() != 7 {
                if sq.file() != 7 { self.pawn_attacks[Color::White.idx()][i].set_bit(i - 7); }
                if sq.file() != 0 { self.pawn_attacks[Color::White.idx()][i].set_bit(i - 9); }
            }
            if sq.rank() != 0 {
                if sq.file() != 7 { self.pawn_attacks[Color::Black.idx()][i].set_bit(i + 9); }
                if sq.file() != 0 { self.pawn_attacks[Color::Black.idx()][i].set_bit(i + 7); }
            }
        }

        // slider range
        for start in 0..64 {
            for end in 0..64 {
                let min = std::cmp::min(start, end);
                let max = std::cmp::max(start, end);

                if max != min {
                    let min_sq = Square::from_usize(min);
                    let max_sq = Square::from_usize(max);

                    let min_r = min_sq.rank() as i16;
                    let min_f = min_sq.file() as i16;

                    let max_r = max_sq.rank() as i16;
                    let max_f = max_sq.file() as i16;

                    let abs_rank_diff = (max_r - min_r).abs();
                    let abs_file_diff = (max_f - min_f).abs();

                    // vertical
                    if min_f == max_f {
                        let mut i =  min + 8;
                        while i < max {
                            self.slider_range[start][end].set_bit(i);
                            i += 8;
                        }
                    }

                    // horizontal
                    else if min_r == max_r {
                        let mut i = min+1;
                        while i < max {
                            self.slider_range[start][end].set_bit(i);
                            i += 1;
                        }
                    }
                    
                    // diagonal
                    else if abs_rank_diff == abs_file_diff {
                        // left to right (downwards) diagonal
                        if max_r - min_r == max_f - min_f {
                            let mut r = min_r - 1;
                            let mut f = min_f - 1;

                            while Square::valid_rf(r, f) && r > max_r && f > max_f {
                                let pos =  Square::from_rf(r as usize, f as usize) as usize;
                                self.slider_range[start][end].set_bit(pos);
                                r -= 1;
                                f -= 1;
                            }
                        }
                        // right to left (downwards) diagonal
                        else {
                            let mut r = min_r - 1;
                            let mut f = min_f + 1;
                            
                            while Square::valid_rf(r, f) && r > max_r && f < max_f {
                                let pos =  Square::from_rf(r as usize, f as usize) as usize;
                                self.slider_range[start][end].set_bit(pos);
                                r -= 1;
                                f += 1;
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
        
            rook_moves: [[0; 4096]; 64],
            bishop_moves: [[0; 4096]; 64],
        
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
    
}
