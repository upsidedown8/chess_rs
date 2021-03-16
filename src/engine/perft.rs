use crate::engine::board::Board;
use crate::engine::r#move::{UndoInfo,MoveUtils};
use crate::engine::movegen::{MoveList,MoveGenerator};

pub fn perft(depth: usize, board: &mut Board, move_generator: &MoveGenerator, move_lists: &mut Vec<MoveList>) -> u64 {
    move_generator.gen_moves(board, &mut move_lists[depth - 1]);

    if depth <= 1 {
        return move_lists[depth - 1].len() as u64;
    }

    let mut nodes = 0;
    let mut info = UndoInfo::default();

    for i in 0..move_lists[depth - 1].len() {
        let current_move = move_lists[depth - 1].at(i);

        board.make_move(current_move, &mut info);
        nodes += perft(depth - 1, board, move_generator, move_lists);
        board.undo_move(current_move, &info);
    }

    nodes
}

pub fn perft_divide(depth: usize, board: &mut Board) -> u64 {
    let move_generator = MoveGenerator::new();
    let mut move_lists = Vec::new();

    for _ in 0..depth {
        move_lists.push(MoveList::new());
    }

    move_generator.gen_moves(board, &mut move_lists[depth - 1]);

    let mut nodes = 0;
    let mut info = UndoInfo::default();

    for i in 0..move_lists[depth - 1].len() {
        let current_move = move_lists[depth - 1].at(i);

        board.make_move(current_move, &mut info);
        let inner_nodes = if depth <= 1 { 1 } else { perft(depth - 1, board, &move_generator, &mut move_lists) };
        println!("{}: {}", current_move.move_to_string(), inner_nodes);
        board.undo_move(current_move, &info);

        nodes += inner_nodes;
    }

    println!("\nNodes searched: {}", nodes);

    nodes
}

#[cfg(test)]
mod tests {
    fn perft_test(fen: &str, depth: usize) -> u64 {
        use super::*;

        let move_generator = MoveGenerator::new();
        let mut move_lists = Vec::new();
        let mut board = Board::new(fen);
    
        for _ in 0..depth {
            move_lists.push(MoveList::new());
        }

        perft(depth, &mut board, &move_generator, &mut move_lists)
    }

    #[test]
    fn perft_depth_1() {
        assert_eq!(perft_test("r6r/1b2k1bq/8/8/7B/8/8/R3K2R b QK - 3 2", 1), 8);
        // assert_eq!(perft_test("8/8/8/2k5/2pP4/8/B7/4K3 b - d3 5 3", 1), 8);
        // assert_eq!(perft_test("r1bqkbnr/pppppppp/n7/8/8/P7/1PPPPPPP/RNBQKBNR w QqKk - 2 2", 1), 19);
        // assert_eq!(perft_test("r3k2r/p1pp1pb1/bn2Qnp1/2qPN3/1p2P3/2N5/PPPBBPPP/R3K2R b QqKk - 3 2", 1), 5);
        // assert_eq!(perft_test("rnb2k1r/pp1Pbppp/2p5/q7/2B5/8/PPPQNnPP/RNB1K2R w QK - 3 9", 1), 39);
        // assert_eq!(perft_test("2r5/3pk3/8/2P5/8/2K5/8/8 w - - 5 4", 1), 9);
        // assert_eq!(perft_test("2kr3r/p1ppqpb1/bn2Qnp1/3PN3/1p2P3/2N5/PPPBBPPP/R3K2R b QK - 3 2", 1), 44);
        // assert_eq!(perft_test("4k3/8/8/5R2/8/8/8/4K3 b - - 0 1", 1), 3);
        // assert_eq!(perft_test("8/4k3/8/8/4R3/8/8/4K3 b - - 0 1", 1), 6);
        // assert_eq!(perft_test("4k3/6N1/5b2/4R3/8/8/8/4K3 b - - 0 1", 1), 4);
        // assert_eq!(perft_test("4k3/8/6n1/4R3/8/8/8/4K3 b - - 0 1", 1), 6);
        // assert_eq!(perft_test("8/8/8/2k5/3Pp3/8/8/4K3 b - d3 0 1", 1), 9);
        // assert_eq!(perft_test("8/8/8/1k6/3Pp3/8/8/4KQ2 b - d3 0 1", 1), 6);
        // assert_eq!(perft_test("4k3/8/4r3/8/8/4Q3/8/2K5 b - - 0 1", 1), 9);
        // assert_eq!(perft_test("8/8/8/8/k2Pp2Q/8/8/2K5 b - d3 0 1", 1), 6);
        // assert_eq!(perft_test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 1), 44);
        // assert_eq!(perft_test("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 1), 46);
        // assert_eq!(perft_test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 1), 6);
        // assert_eq!(perft_test("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 1), 6);
        // assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 1), 14);
        // assert_eq!(perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 1), 48);
        // assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 1), 20);
    }
    #[test]
    fn perft_depth_2() {
        assert_eq!(perft_test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 2), 1486);
        assert_eq!(perft_test("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 2), 2079);
        assert_eq!(perft_test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 2), 264);
        assert_eq!(perft_test("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 2), 264);
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - -", 2), 191);
        assert_eq!(perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -", 2), 2039);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 2), 400);
    }
    #[test]
    fn perft_depth_3() {
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 3), 2812);
        assert_eq!(perft_test("r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 3), 89890);
        assert_eq!(perft_test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 3), 9467);
        assert_eq!(perft_test("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 3), 9467);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 3), 8902);
        assert_eq!(perft_test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 3), 62379);
        assert_eq!(perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 3), 97862);
    }
    #[test]
    fn perft_depth_4() {
        assert_eq!(perft_test("r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1", 4), 1274206);
        assert_eq!(perft_test("r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1", 4), 1720476);
        assert_eq!(perft_test("8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1", 4), 23527);
        assert_eq!(perft_test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 4), 422333);
        assert_eq!(perft_test("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 4), 422333);
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 4), 43238);
        assert_eq!(perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 4), 4085603);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 4), 197281);
        assert_eq!(perft_test("r3k2r/p1ppqpb1/bn1Ppnp1/4N3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R b KQkq - 0 1", 4), 3835265);
    }
    #[test]
    fn perft_depth_5() {
        assert_eq!(perft_test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 5), 15833292);
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 5), 674624);
        assert_eq!(perft_test("8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1", 5), 1004658);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 5), 4865609);
        assert_eq!(perft_test("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 5), 15833292);
        assert_eq!(perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 5), 193690690);
    }
    #[test]
    fn perft_depth_6() {
        assert_eq!(perft_test("3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1", 6), 1134888);
        assert_eq!(perft_test("8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1", 6), 1015133);
        assert_eq!(perft_test("8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1", 6), 1440467);
        assert_eq!(perft_test("5k2/8/8/8/8/8/8/4K2R w K - 0 1", 6), 661072);
        assert_eq!(perft_test("3k4/8/8/8/8/8/8/R3K3 w Q - 0 1", 6), 803711);
        assert_eq!(perft_test("2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1", 6), 3821001);
        assert_eq!(perft_test("4k3/1P6/8/8/8/8/K7/8 w - - 0 1", 6), 217342);
        assert_eq!(perft_test("8/P1k5/K7/8/8/8/8/8 w - - 0 1", 6), 92683);
        assert_eq!(perft_test("K1k5/8/P7/8/8/8/8/8 w - - 0 1", 6), 2217);
        assert_eq!(perft_test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 6), 706045033);
        assert_eq!(perft_test("r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1", 6), 706045033);
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 6), 11030083);
        assert_eq!(perft_test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 6), 8031647685);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 6), 119060324);
    }
    #[test]
    fn perft_depth_7() {
        assert_eq!(perft_test("8/k1P5/8/1K6/8/8/8/8 w - - 0 1", 7), 567584);
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 7), 178633661);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 7), 3195901860);
    }
    #[test]
    fn perft_depth_8() {
        assert_eq!(perft_test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 8), 3009794393);
        assert_eq!(perft_test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 8), 84998978956);
    }
}
