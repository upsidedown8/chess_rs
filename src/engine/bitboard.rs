/*

 a8 => lsb (index 0)
 h1 => msb (index 63)

    a b c d e f g h
  ╭─────────────────╮
8 │ x x x x x x x x │ 8
7 │ x x x x x x x x │ 7
6 │ x x x x x x x x │ 6
5 │ x x x x x x x x │ 5
4 │ x x x x x x x x │ 4
3 │ x x x x x x x x │ 3
2 │ x x x x x x x x │ 2
1 │ x x x x x x x x │ 1
  ╰─────────────────╯
    a b c d e f g h

*/

const LSB_64_TABLE: [usize; 64] = [
    63, 30,  3, 32, 25, 41, 22, 33,
    15, 50, 42, 13, 11, 53, 19, 34,
    61, 29,  2, 51, 21, 43, 45, 10,
    18, 47,  1, 54,  9, 57,  0, 35,
    62, 31, 40,  4, 49,  5, 52, 26,
    60,  6, 23, 44, 46, 27, 56, 16,
     7, 39, 48, 24, 59, 14, 12, 55,
    38, 28, 58, 20, 37, 17, 36,  8
];
pub const FULL_BB: u64 = 0xffff_ffff_ffff_ffff;

pub trait BitBoardUtils {
    fn pop_lsb(&mut self) -> usize;
    fn lsb_idx(&self) -> usize;

    fn count_1s(&self) -> usize;

    fn set_bit(&mut self, idx: usize);
    fn clear_bit(&mut self, idx: usize);

    fn is_bit_set(&self, idx: usize) -> bool;

    fn bb_to_string(&self) -> String;
}

impl BitBoardUtils for u64 {
    #[inline(always)]
    fn pop_lsb(&mut self) -> usize {
        debug_assert!(*self != 0);
        let b = *self ^ (*self - 1);
        let folded = (b & 0xffffffff) ^ (b >> 32);
        *self &= *self - 1;
        let idx = ((folded * 0x783A9B23) >> 26) as usize;
        LSB_64_TABLE[idx]
    }

    #[inline(always)]
    fn lsb_idx(&self) -> usize {
        debug_assert!(*self != 0);
        let b = *self ^ (*self - 1);
        let folded = (b & 0xffffffff) ^ (b >> 32);
        let idx = ((folded * 0x783A9B23) >> 26) as usize;
        LSB_64_TABLE[idx]
    }

    #[inline(always)]
    fn count_1s(&self) -> usize {
        let mut tmp = *self;
        let mut count = 0;
        while tmp != 0 {
            tmp &= tmp - 1;
            count += 1;
        }
        count
    }

    #[inline(always)]
    fn set_bit(&mut self, idx: usize) {
        *self |= 1 << idx;
    }

    #[inline(always)]
    fn clear_bit(&mut self, idx: usize) {
        *self &= !(1 << idx);
    }

    #[inline(always)]
    fn is_bit_set(&self, idx: usize) -> bool {
        *self & (1 << idx) != 0
    }

    fn bb_to_string(&self) -> String {
        let mut result = concat!(
            "    a b c d e f g h\n",
            "  ╭─────────────────╮\n"
        ).to_string();

        for rank in 0..8 {
            result.push_str(&format!("{} │ ", 8 - rank));
            for file in 0..8 {
                result.push_str(&format!(
                    "{} ",
                    if self.is_bit_set(rank * 8 + file) {
                        'x'
                    } else {
                        '.'
                    }
                ));
            }
            result.push_str(&format!("│ {}\n", 8 - rank));
        }

        result.push_str(concat!(
            "  ╰─────────────────╯\n",
            "    a b c d e f g h"
        ));

        result
    }
}
