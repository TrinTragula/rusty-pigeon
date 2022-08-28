#[cfg(test)]
mod static_evaluation_tests {
    use crate::{
        board::{fen::FenParser, models::Engine},
        constants::START_POS,
        evaluate::evaluator::Evaluate,
    };

    #[test]
    fn start_pos_static_evaluation_is_zero() {
        let mut engine = Engine::from_position(FenParser::fen_to_position(START_POS));
        assert_eq!(Evaluate::static_evaluation(&mut engine), 0);
    }

    #[test]
    fn start_pos_static_evaluation_is_ok_with_one_piece_eaten() {
        let fen = "rnbqkbnr/pppp1ppp/8/8/8/8/PPPPPPPP/RNBQKBNR b - - 0 1";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        assert!(Evaluate::static_evaluation(&mut engine) < 0);
    }
}

#[cfg(test)]
mod ponder_tests {
    use crate::{
        board::{
            fen::FenParser,
            models::{Engine, Move, Piece, Square},
        },
        evaluate::evaluator::{Evaluate}, uci::options::GoOptions,
    };

    #[test]
    fn should_promote_to_queen() {
        let fen = "rnbqkbnr/1ppppppp/8/8/P3P3/8/1p1PKPPP/RN1Q1BNR b kq - 1 6";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Promotion(Square::B2, Square::A1, Piece::QUEEN)
        );
    }

    #[test]
    #[ignore = "takes a lot of time"]
    fn start_pos() {
        let mut engine = Engine::from_position(FenParser::fen_to_position(crate::constants::START_POS));
        Evaluate::search(&mut engine, GoOptions::depth(7), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Normal(Square::E2, Square::E4)
        );
    }

    #[test]
    fn mate_in_1() {
        let fen = "4k1B1/P6R/8/1P1P4/5P1P/3P1P1P/8/R3K3 w Q - 1 43";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Promotion(Square::A7, Square::A8, Piece::QUEEN)
        );
        let fen = "r1b2b1r/pp3Qp1/2nkn2p/3ppP1p/P1p5/1NP1NB2/1PP1PPR1/1K1R3q w - - 0 1";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Normal(Square::E3, Square::C4)
        );
        let fen = "rnbqkbnr/pp1p1ppp/2p5/8/8/8/PPPPP2P/RNBQKBNR b KQkq - 0 1";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Normal(Square::D8, Square::H4)
        );
    }

    #[test]
    fn mate_in_2() {
        let fen = "2bqkbn1/2pppp2/np2N3/r3P1p1/p2N2B1/5Q2/PPPPKPP1/RNB2r2 w KQkq - 0 1";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Normal(Square::F3, Square::F7)
        );

        let fen = "r2qk2r/pb4pp/1n2Pb2/2B2Q2/p1p5/2P5/2B2PPP/RN2R1K1 w - - 1 1";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Normal(Square::F5, Square::G6)
        );

        let fen = "6k1/pp4p1/2p5/2bp4/8/P5Pb/1P3rrP/2BRRN1K b - - 0 1";
        let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
        Evaluate::search(&mut engine, GoOptions::depth(5), None);
        assert_eq!(
            engine.current_best_move[0].clone().unwrap().m,
            Move::Normal(Square::G2, Square::G1)
        );
    }
}

#[cfg(test)]
mod uci_options_parser_tests {
    use crate::{
        board::models::Side, uci::options::GoOptions,
    };

    #[test]
    fn parse_correctly() {
        let sut = GoOptions::parse("go infinite");
        assert_eq!(sut.wtime, None);
        assert_eq!(sut.btime, None);
        assert_eq!(sut.winc, None);
        assert_eq!(sut.binc, None);
        assert_eq!(sut.movestogo, None);
        assert_eq!(sut.depth, None);
        assert_eq!(sut.infinite, true);

        let sut = GoOptions::parse("go depth 5");
        assert_eq!(sut.wtime, None);
        assert_eq!(sut.btime, None);
        assert_eq!(sut.winc, None);
        assert_eq!(sut.binc, None);
        assert_eq!(sut.movestogo, None);
        assert_eq!(sut.depth, Some(5));
        assert_eq!(sut.infinite, false);

        let sut = GoOptions::parse("go wtime 300000 btime 300001 movestogo 40");
        assert_eq!(sut.wtime, Some(300000));
        assert_eq!(sut.btime, Some(300001));
        assert_eq!(sut.winc, None);
        assert_eq!(sut.binc, None);
        assert_eq!(sut.movestogo, Some(40));
        assert_eq!(sut.depth, None);
        assert_eq!(sut.infinite, false);

        let sut = GoOptions::parse("go wtime 1234 btime 1235 winc 33 binc 35");
        assert_eq!(sut.wtime, Some(1234));
        assert_eq!(sut.btime, Some(1235));
        assert_eq!(sut.winc, Some(33));
        assert_eq!(sut.binc, Some(35));
        assert_eq!(sut.movestogo, None);
        assert_eq!(sut.depth, None);
        assert_eq!(sut.infinite, false);
    }

    #[test]
    fn interpret_options_correctly() {
        let (depth, max_time_in_ms) =
            GoOptions::parse_uci_options(GoOptions::parse("go infinite"), Side::WHITE, 0);
        assert_eq!(depth, 99);
        assert_eq!(max_time_in_ms, None);

        let (depth, max_time_in_ms) =
            GoOptions::parse_uci_options(GoOptions::parse("go depth 5"), Side::WHITE, 0);
        assert_eq!(depth, 5);
        assert_eq!(max_time_in_ms, None);

        let (depth, max_time_in_ms) = GoOptions::parse_uci_options(
            GoOptions::parse("go wtime 300000 btime 300001 movestogo 41"),
            Side::WHITE,
            0,
        );
        assert_eq!(depth, 99);
        assert_eq!(max_time_in_ms, Some(7317));

        let (depth, max_time_in_ms) = GoOptions::parse_uci_options(
            GoOptions::parse("go wtime 1234 btime 1235 winc 33 binc 35"),
            Side::WHITE,
            0,
        );
        assert_eq!(depth, 99);
        assert_eq!(max_time_in_ms, Some(500));
    }
}
