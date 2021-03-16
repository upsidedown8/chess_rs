mod engine;

use engine::board::Board;
use engine::perft::*;

fn main() {
    let mut board = Board::default();

    println!("{}\n{}", board.to_string(), board.to_fen());

    perft_divide(6, &mut board);
}
