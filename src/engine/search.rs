use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::{MoveGenerator, MoveList};
use crate::engine::r#move::{UndoInfo, Move};

pub fn search(
    depth: usize,
    mut alpha: i32,
    beta: i32,
    board: &mut Board,
    evaluator: &Evaluator,
    move_generator: &MoveGenerator,
    move_lists: &mut Vec<MoveList>,
) -> i32 {

    if depth == 0 {
        evaluator.eval(board)
    } else {
        let mut best = i32::MIN+1;

        move_generator.gen_moves(board, &mut move_lists[depth - 1]);

        let num_moves = move_lists[depth - 1].len();

        // check for end of game
        if num_moves == 0 {
            // check for stalemate
            if !move_generator.is_in_check(board) {
                best = 0;
            }

            // otherwise loss
        }
        
        // continue search
        else {
            let mut info = UndoInfo::default();
    
            for i in 0..num_moves {
                let my_move = move_lists[depth - 1].at(i);
    
                // do the move
                board.make_move(my_move, &mut info);
                
                // test the move
                best = std::cmp::max(best, -search(depth - 1, -beta, -alpha, board, evaluator, move_generator, move_lists));

                // update alpha
                alpha = std::cmp::max(alpha, best);
                
                // undo changes
                board.undo_move(my_move, &info);
    
                // alpha/beta cut-off
                if alpha >= beta {
                    break;
                }
            }
        }
        
        best
    }

}

pub fn best_move(
    max_depth: usize,
    board: &mut Board,
    evaluator: &Evaluator,
    move_generator: &MoveGenerator,
    move_lists: &mut Vec<MoveList>,
) -> Option<Move> {
    move_generator.gen_moves(board, &mut move_lists[max_depth - 1]);

    let mut best_move = None;
    let mut best_score = i32::MIN+1;

    let mut info = UndoInfo::default();

    for i in 0..move_lists[max_depth - 1].len() {
        let my_move = move_lists[max_depth - 1].at(i);

        // test the move
        board.make_move(my_move, &mut info);
        let score = -search(max_depth - 1, i32::MIN+1, i32::MAX-1, board, evaluator, move_generator, move_lists);
        board.undo_move(my_move, &info);

        // store the best move
        if score > best_score {
            best_score = score;
            best_move = Some(my_move);
        }
    }

    best_move
}
