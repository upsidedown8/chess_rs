mod engine;

use engine::board::Board;

fn main() {
    let board = Board::default();

    println!("{}", board);
    println!("{}", board.to_fen());
}
