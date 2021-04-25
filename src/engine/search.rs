use crate::engine::board::Board;
use crate::engine::eval::Evaluator;
use crate::engine::movegen::{MoveGenerator, MoveList};
use crate::engine::r#move::{Move, MoveUtils, UndoInfo};

extern crate time;
use time::{Duration, Instant};

pub fn negamax(
    depth: usize,
    mut alpha: i32,
    beta: i32,
    board: &mut Board,
    evaluator: &mut Evaluator,
    move_generator: &MoveGenerator,
    move_lists: &mut Vec<MoveList>,
) -> i32 {
    if depth == 0 {
        evaluator.score(board.friendly_color())
    } else {
        let mut best = i32::MIN + 1;

        // generate and order the moves
        move_generator.gen_moves(board, &mut move_lists[depth - 1]);
        move_lists[depth - 1].order_moves(board);

        let num_moves = move_lists[depth - 1].len();

        // check for end of game
        if num_moves == 0 {
            // check for stalemate
            if !move_generator.is_in_check(board) {
                best = 0;
            }

            // otherwise loss
        }
        // fifty move / low material / threefold repetition
        else if board.is_draw() {
            best = 0;
        } else {
            // continue search
            let mut info = UndoInfo::default();

            for i in 0..num_moves {
                let my_move = move_lists[depth - 1].at(i);

                // do the move
                board.make_move(my_move, &mut info);

                // update evaluation
                evaluator.update_score(info.evalutor_diff);

                // test the move
                best = std::cmp::max(
                    best,
                    -negamax(
                        depth - 1,
                        -beta,
                        -alpha,
                        board,
                        evaluator,
                        move_generator,
                        move_lists,
                    ),
                );

                // update alpha
                alpha = std::cmp::max(alpha, best);

                // undo changes
                board.undo_move(my_move, &info);

                // reset evaluation
                evaluator.update_score(-info.evalutor_diff);

                // alpha/beta cut-off
                if alpha >= beta {
                    break;
                }
            }
        }

        best
    }
}

pub fn find_best_move(
    max_depth: usize,
    board: &mut Board,
    evaluator: &mut Evaluator,
    move_generator: &MoveGenerator,
    move_lists: &mut Vec<MoveList>,
) -> Option<(Move, i32)> {
    // setup evaluator
    evaluator.init_score(board);

    move_generator.gen_moves(board, &mut move_lists[max_depth - 1]);

    let mut best_move = None;
    let mut best_score = i32::MIN + 1;

    let mut info = UndoInfo::default();

    for i in 0..move_lists[max_depth - 1].len() {
        let my_move = move_lists[max_depth - 1].at(i);

        // test the move
        board.make_move(my_move, &mut info);

        // update evaluation
        evaluator.update_score(info.evalutor_diff);

        let score = -negamax(
            max_depth - 1,
            i32::MIN + 1,
            i32::MAX - 1,
            board,
            evaluator,
            move_generator,
            move_lists,
        );

        // undo move
        board.undo_move(my_move, &info);
        
        // update evaluation
        evaluator.update_score(-info.evalutor_diff);

        // store the best move
        if score >= best_score {
            best_score = score;
            best_move = Some(my_move);
        }
    }

    if best_move.is_some() {
        Some((best_move.unwrap(), best_score))
    } else {
        None
    }
}

pub fn iterative_deepening(
    max_depth: usize,
    board: &mut Board,
    evaluator: &mut Evaluator,
    move_generator: &MoveGenerator,
    move_lists: &mut Vec<MoveList>,
    max_time_millis: usize,
) {
    // setup evaluator
    evaluator.init_score(board);

    move_generator.gen_moves(board, &mut move_lists[max_depth - 1]);

    let mut best_move = 0;

    let start = Instant::now();

    for depth in 1..=max_depth {
        // calculate score
        let (my_move, score) = find_best_move(
            max_depth,
            board,
            evaluator,
            move_generator,
            move_lists,
        ).unwrap();

        best_move = my_move;

        let end = Instant::now();

        let millis: usize = (end - start).whole_milliseconds() as usize;
        
        // output pv line
        println!(
            "info score cp {} depth {} move {} time {}",
            score,
            depth,
            best_move.move_to_string(),
            millis,
        );

        // check for out of time
        if millis >= max_time_millis {
            break;
        }
    }

}
