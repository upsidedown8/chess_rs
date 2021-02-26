#[repr(usize)]
#[derive(Clone, Copy, PartialEq)]
pub enum Square {
    // LSB (0) = A8
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
    // MSB (63) = H1
}

impl Square {
    #[inline(always)]
    pub fn sq(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    pub fn rank(&self) -> usize {
        self.sq() / 8
    }

    #[inline(always)]
    pub fn file(&self) -> usize {
        self.sq() % 8
    }

    pub fn notation(&self) -> String {
        format!("{}{}", char::from(97 + self.file() as u8), 8 - self.rank())
    }

    #[inline(always)]
    pub fn from_rf(rank: usize, file: usize) -> Square {
        let idx = rank * 8 + file;
        debug_assert!(Square::valid_rf(rank as i16, file as i16));
        debug_assert!(idx < 64);
        Square::from_usize(idx)
    }

    pub fn from_notation(string: &str) -> Square {
        let file = "abcdefgh"
            .chars()
            .position(|x| x == string.chars().next().unwrap())
            .unwrap();
        let rank = string.chars().nth(1).unwrap().to_digit(10).unwrap();
        Square::from_rf(8 - rank as usize, file)
    }

    pub fn from_usize(idx: usize) -> Square {
        use Square::*;
        const SQUARES: [Square; 64] = [
            A8, B8, C8, D8, E8, F8, G8, H8,
            A7, B7, C7, D7, E7, F7, G7, H7,
            A6, B6, C6, D6, E6, F6, G6, H6,
            A5, B5, C5, D5, E5, F5, G5, H5,
            A4, B4, C4, D4, E4, F4, G4, H4,
            A3, B3, C3, D3, E3, F3, G3, H3,
            A2, B2, C2, D2, E2, F2, G2, H2,
            A1, B1, C1, D1, E1, F1, G1, H1,
        ];
        debug_assert!(idx < 64);
        SQUARES[idx]
    }

    pub fn valid_rf(rank: i16, file: i16) -> bool {
        (0..8).contains(&rank) && (0..8).contains(&file)
    }

    pub fn valid_sq(sq: i16) -> bool {
        (0..64).contains(&sq)
    }
}
