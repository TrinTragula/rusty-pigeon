#[cfg(test)]
mod movegen_tests {
    use crate::{
        board::{
            fen::FenParser,
            models::{Engine, Move, Square, Piece},
        },
        constants::START_POS,
        movegen::generator::{MoveGenKind, MoveGenerator, MoveInfo},
    };

    #[test]
    fn start_pos_generate_right_no_of_moves() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert_eq!(moves.len(), 20);

        let m = MoveInfo {
            m: Move::Normal(Square::E2, Square::E4),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m);
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert_eq!(moves.len(), 20);

        sut.undo_move(&m);
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert_eq!(moves.len(), 20);
    }

    #[test]
    fn check_castling_is_available() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        sut.apply_algebraic_move("e7e5");
        sut.apply_algebraic_move("g1f3");
        sut.apply_algebraic_move("g8f6");
        sut.apply_algebraic_move("f1e2");
        sut.apply_algebraic_move("f8e7");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "e1g1"));
        sut.apply_algebraic_move("a2a3");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "e8g8"));
    }

    #[test]
    fn check_rook() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("a2a4");
        sut.apply_algebraic_move("a7a5");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "a1a2"));
        assert!(moves.iter().any(|m| format!("{m}") == "a1a3"));
        sut.apply_algebraic_move("a1a2");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "a8a7"));
        assert!(moves.iter().any(|m| format!("{m}") == "a8a6"));
    }

    #[test]
    fn check_bishop() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        sut.apply_algebraic_move("e7e5");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "f1e2"));
        assert!(moves.iter().any(|m| format!("{m}") == "f1d3"));
        assert!(moves.iter().any(|m| format!("{m}") == "f1c4"));
        assert!(moves.iter().any(|m| format!("{m}") == "f1b5"));
        assert!(moves.iter().any(|m| format!("{m}") == "f1a6"));
        sut.apply_algebraic_move("a2a3");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "f8e7"));
        assert!(moves.iter().any(|m| format!("{m}") == "f8d6"));
        assert!(moves.iter().any(|m| format!("{m}") == "f8c5"));
        assert!(moves.iter().any(|m| format!("{m}") == "f8b4"));
        assert!(moves.iter().any(|m| format!("{m}") == "f8a3"));
    }

    #[test]
    fn check_queen() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        sut.apply_algebraic_move("e7e5");
        sut.apply_algebraic_move("d1g4");
        sut.apply_algebraic_move("d8g5");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "g4g5"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4f5"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4e6"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4d7"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4h5"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4f4"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4h4"));
        assert!(moves.iter().any(|m| format!("{m}") == "g4g3"));
        sut.apply_algebraic_move("a2a3");
        let moves = MoveGenerator::get_pseudo_legal_moves(&sut.position, &MoveGenKind::All);
        assert!(moves.iter().any(|m| format!("{m}") == "g5g4"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5f4"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5e3"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5d2"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5f5"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5h4"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5h5"));
        assert!(moves.iter().any(|m| format!("{m}") == "g5g6"));
    }
}

#[cfg(test)]
mod perft_tests {
    // Positions and values taken from https://www.chessprogramming.org/Perft_Results
    use crate::{
        board::{fen::FenParser, models::Engine, utils::perft},
        constants::START_POS,
    };

    #[test]
    fn check_perft_from_start_pos() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 20);
        assert_eq!(perft(&mut sut, 2, true, false, true), 400);
        assert_eq!(perft(&mut sut, 3, true, false, true), 8902);
        assert_eq!(perft(&mut sut, 4, true, false, true), 197281);
        // This takes a lot
        // assert_eq!(perft(&mut sut, 5, true, false, true), 4865609);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 119060324);

        // Never done this
        // assert_eq!(perft(&mut sut, 7, true, false, true), 3195901860);
        // assert_eq!(perft(&mut sut, 8, true, false, true), 84998978956);
        // assert_eq!(perft(&mut sut, 9, true, false, true), 2439530234167);
        // assert_eq!(perft(&mut sut, 10, true, false, true), 69352859712417);
        // assert_eq!(perft(&mut sut, 11, true, false, true), 2097651003696806);
        // assert_eq!(perft(&mut sut, 12, true, false, true), 62854969236701747);
        // assert_eq!(perft(&mut sut, 13, true, false, true), 1981066775000396239);
        // assert_eq!(perft(&mut sut, 14, true, false, true), 61885021521585529237);
        // assert_eq!(perft(&mut sut, 15, true, false, true), 2015099950053364471960);
    }

    #[test]
    fn check_perft_from_kiwipete() {
        let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 48);
        assert_eq!(perft(&mut sut, 2, true, false, true), 2039);
        assert_eq!(perft(&mut sut, 3, true, false, true), 97862);
        assert_eq!(perft(&mut sut, 4, true, false, true), 4085603);
        // This takes a lot
        // assert_eq!(perft(&mut sut, 5, true, false, true), 193690690);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 8031647685);
    }

    #[test]
    fn check_perft_from_position_3() {
        let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 14);
        assert_eq!(perft(&mut sut, 2, true, false, true), 191);
        assert_eq!(perft(&mut sut, 3, true, false, true), 2812);
        assert_eq!(perft(&mut sut, 4, true, false, true), 43238);
        assert_eq!(perft(&mut sut, 5, true, false, true), 674624);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 11030083);
        // assert_eq!(perft(&mut sut, 7, true, false, true), 178633661);
        // assert_eq!(perft(&mut sut, 8, true, false, true), 3009794393);
    }

    #[test]
    fn check_perft_from_position_4() {
        let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 6);
        assert_eq!(perft(&mut sut, 2, true, false, true), 264);
        assert_eq!(perft(&mut sut, 3, true, false, true), 9467);
        assert_eq!(perft(&mut sut, 4, true, false, true), 422333);
        // assert_eq!(perft(&mut sut, 5, true, false, true), 15833292);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 706045033);
    }

    #[test]
    fn check_perft_from_position_4_mirrored() {
        let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 6);
        assert_eq!(perft(&mut sut, 2, true, false, true), 264);
        assert_eq!(perft(&mut sut, 3, true, false, true), 9467);
        assert_eq!(perft(&mut sut, 4, true, false, true), 422333);
        // assert_eq!(perft(&mut sut, 5, true, false, true), 15833292);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 706045033);
    }

    #[test]
    fn check_perft_from_position_5() {
        let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 44);
        assert_eq!(perft(&mut sut, 2, true, false, true), 1486);
        assert_eq!(perft(&mut sut, 3, true, false, true), 62379);
        assert_eq!(perft(&mut sut, 4, true, false, true), 2103487);
        // assert_eq!(perft(&mut sut, 5, true, false, true), 89941194);
    }

    #[test]
    fn check_perft_from_position_last() {
        let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 46);
        assert_eq!(perft(&mut sut, 2, true, false, true), 2079);
        assert_eq!(perft(&mut sut, 3, true, false, true), 89890);
        assert_eq!(perft(&mut sut, 4, true, false, true), 3894594);
        // assert_eq!(perft(&mut sut, 5, true, false, true), 164075551);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 6923051137);
        // assert_eq!(perft(&mut sut, 7, true, false, true), 287188994746);
        // assert_eq!(perft(&mut sut, 8, true, false, true), 11923589843526);
        // assert_eq!(perft(&mut sut, 9, true, false, true), 490154852788714);
    }

    #[test]
    fn check_perft_promotion() {
        let fen = "n1n5/PPPk4/8/8/8/8/4Kppp/5N1N b - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 1, true, false, true), 24);
        assert_eq!(perft(&mut sut, 2, true, false, true), 496);
        assert_eq!(perft(&mut sut, 3, true, false, true), 9483);
        assert_eq!(perft(&mut sut, 4, true, false, true), 182838);
        assert_eq!(perft(&mut sut, 5, true, false, true), 3605103);
        // assert_eq!(perft(&mut sut, 6, true, false, true), 71179139);
    }

    #[test]
    fn check_perft_illegal_ep_move_1() {
        let fen = "3k4/3p4/8/K1P4r/8/8/8/8 b - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 1134888);
    }

    #[test]
    fn check_perft_illegal_ep_move_2() {
        let fen = "8/8/4k3/8/2p5/8/B2P2K1/8 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 1015133);
    }

    #[test]
    fn check_perft_en_passant_checks() {
        let fen = "8/8/1k6/2b5/2pP4/8/5K2/8 b - d3 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 1440467);
    }

    #[test]
    fn check_perft_short_castle_check() {
        let fen = "5k2/8/8/8/8/8/8/4K2R w K - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 661072);
    }

    #[test]
    fn check_perft_long_castle_check() {
        let fen = "3k4/8/8/8/8/8/8/R3K3 w Q - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 803711);
    }

    #[test]
    fn check_perft_castle_rights() {
        let fen = "r3k2r/1b4bq/8/8/8/8/7B/R3K2R w KQkq - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 4, true, false, true), 1274206);
    }

    #[test]
    fn check_perft_castling_prevented() {
        let fen = "r3k2r/8/3Q4/8/8/5q2/8/R3K2R b KQkq - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 4, true, false, true), 1720476);
    }

    #[test]
    fn check_perft_promote_out_of_check() {
        let fen = "2K2r2/4P3/8/8/8/8/8/3k4 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 3821001);
    }

    #[test]
    fn check_perft_discovered_check() {
        let fen = "8/8/1P2K3/8/2n5/1q6/8/5k2 b - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 5, true, false, true), 1004658);
    }

    #[test]
    fn check_perft_promote_to_give_check() {
        let fen = "4k3/1P6/8/8/8/8/K7/8 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 217342);
    }

    #[test]
    fn check_perft_under_promote_to_give_check() {
        let fen = "8/P1k5/K7/8/8/8/8/8 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 92683);
    }

    #[test]
    fn check_perft_self_stalemate() {
        let fen = "K1k5/8/P7/8/8/8/8/8 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 6, true, false, true), 2217);
    }

    #[test]
    fn check_perft_stalemate_and_checkmate() {
        let fen = "8/k1P5/8/1K6/8/8/8/8 w - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 7, true, false, true), 567584);
    }

    #[test]
    fn check_perft_stalemate_and_checkmate_2() {
        let fen = "8/8/2k5/5q2/5n2/8/5K2/8 b - - 0 1";
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(perft(&mut sut, 4, true, false, true), 23527);
    }
}
