use crate::engine::perft;
use crate::engine::r#move::{MoveUtils, UndoInfo};
use crate::engine::search;
use crate::engine::{
    board::Board,
    eval::Evaluator,
    movegen::{MoveGenerator, MoveList},
};

const MAX_DEPTH: usize = 6;

fn parse_moves(
    board: &mut Board,
    tokens: &[&str],
    move_generator: &MoveGenerator,
    start_idx: usize,
) {
    let mut move_list = MoveList::new();
    let mut info = UndoInfo::default();

    // skip (start_idx + 1) to skip previous tokens and "moves" token
    'outer: for &token in tokens.iter().skip(start_idx + 1) {
        move_generator.gen_moves(board, &mut move_list);
        for i in 0..move_list.len() {
            if move_list.at(i).move_to_string().eq(token) {
                board.make_move(move_list.at(i), &mut info);
                continue 'outer;
            }
        }

        // if move wasn't found then stop making moves
        break;
    }
}

pub fn uci() {
    // setup
    let move_generator = MoveGenerator::new();
    let mut evaluator = Evaluator::default();
    let mut move_lists = Vec::new();
    for _ in 0..MAX_DEPTH {
        move_lists.push(MoveList::new());
    }
    let mut board = Board::default();

    loop {
        let mut line_str = String::new();
        std::io::stdin().read_line(&mut line_str).unwrap();

        let tokens = line_str.split_whitespace().collect::<Vec<&str>>();

        if tokens.is_empty() {
            continue;
        }

        match tokens[0] {
            "isready" => {
                println!("readyok");
            }
            "ucinewgame" => {
                board.reset();
            }
            "uci" => {
                println!("id name Avocado");
                println!("id author upsidedown8");
                println!("uciok")
            }
            "quit" => {
                break;
            }
            "d" => {
                println!("{}", board.to_string());
                println!("fen: {}", board.to_fen());
            }
            "position" => {
                if tokens.len() >= 2 {
                    match tokens[1] {
                        "fen" => {
                            // fen: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
                            // tokens:                   1                      2   3  4 5 6

                            if tokens.len() >= 8 {
                                let fen = tokens
                                    .iter()
                                    .skip(2)
                                    .take(6)
                                    .fold(String::new(), |acc, &s| acc + s + " ");
                                let tmp_board = board;

                                if let Err(..) = board.load_fen(&fen) {
                                    // fix any changes
                                    board = tmp_board;
                                    continue;
                                };

                                if tokens.len() >= 9 {
                                    parse_moves(&mut board, &tokens, &move_generator, 8);
                                }
                            }
                        }
                        "startpos" => {
                            board.reset();

                            // ie. contains moves ...
                            if tokens.len() >= 3 {
                                parse_moves(&mut board, &tokens, &move_generator, 2);
                            }
                        }
                        _ => {}
                    }
                }
            }
            "go" => {
                if tokens.len() >= 3 && tokens[1].eq("perft") {
                    let depth = match str::parse::<usize>(tokens[2]) {
                        Ok(d) => d,
                        _ => continue,
                    };

                    perft::perft_divide(depth, &mut board);
                } else {
                    let mut depth = 6;

                    // parse command
                    let mut i = 1;

                    while i < tokens.len() {
                        match tokens[i] {
                            "depth" => {
                                depth = tokens[i + 1].parse().unwrap();
                            },
                            _ => {},
                        }
                        i += 1;
                    }

                    if let Some((best_move, _)) = search::find_best_move(
                        depth,
                        &mut board,
                        &mut evaluator,
                        &move_generator,
                        &mut move_lists,
                    ) {
                        println!("bestmove {}", best_move.move_to_string());
                    }
                }
            }
            _ => {}
        }
    }
}
