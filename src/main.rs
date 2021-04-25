mod engine;

use engine::{board::Board, eval::Evaluator, movegen::{MoveGenerator, MoveList}};
use engine::search;
use engine::r#move::{MoveUtils, UndoInfo};
use engine::perft;
use engine::uci;


fn two_player_console() {
    const MAX_DEPTH: usize = 6;

    // setup
    let move_generator = MoveGenerator::new();
    let mut evaluator = Evaluator::default();
    let mut move_lists = Vec::new();
    for _ in 0..MAX_DEPTH {
        move_lists.push(MoveList::new());
    }

    println!("Enter fen: ");
    let mut fen = String::new();
    std::io::stdin().read_line(&mut fen).expect("Failed to read fen");

    let mut board = Board::new(&fen).expect("Invalid fen");

    perft::perft_divide(MAX_DEPTH, &mut board);

    let mut info = UndoInfo::default();
    
    loop {
        println!("{}\n{}", board.to_string(), board.to_fen());
        
        let mut possible_moves = MoveList::new();
        move_generator.gen_moves(&mut board, &mut possible_moves);

        if possible_moves.len() != 0 {
            let mut my_move = 0;

            if board.friendly_color().is_white() {
                let mut msg = String::new();
                std::io::stdin().read_line(&mut msg).unwrap();

                for i in 0..possible_moves.len() {
                    if possible_moves.at(i).move_to_string() == msg.trim() {
                        my_move = possible_moves.at(i);
                        break;
                    }
                }

                if my_move == 0 {
                    panic!("Illegal move entered");
                }
            } else if let Some((best_move, _)) = search::find_best_move(MAX_DEPTH, &mut board, &mut evaluator, &move_generator, &mut move_lists) {
                my_move = best_move;
            }
            
            println!("{}", my_move.move_to_string());
            board.make_move(my_move, &mut info);
        } else {
            if move_generator.is_in_check(&mut board) {
                println!("Winner: {}", board.enemy_color());
            } else {
                println!("Stalemate");
            }
            
            break;
        }
    }
}

fn main() {
    uci::uci();
}
