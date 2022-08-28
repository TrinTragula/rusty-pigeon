#[cfg(test)]
mod fen_tests {
    use crate::{
        board::{
            fen::FenParser,
            models::{Castling, Piece, Side, Square, Engine},
        },
        constants::START_POS,
    };

    #[test]
    fn parse_start_pos() {
        let fen = START_POS;
        let sut = Engine::from_position(FenParser::fen_to_position(fen));
        assert_eq!(sut.position.side_to_move, Side(Side::WHITE));
        assert_eq!(sut.position.state.castling.0, Castling::ALL);
        assert_eq!(sut.position.state.en_passant.0, Square::NONE);
        assert_eq!(sut.position.half_move_number, 0);
        assert_eq!(sut.position.state.since_last_capture, 0);
        assert_eq!(sut.position.board.side_pieces[Side::WHITE].0, 0b1111111111111111);
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            0b1111111111111111 << 48
        );
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::PAWN].0,
            0b1111111100000000
        );
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::PAWN].0,
            0b0000000011111111 << 48
        );
    }

    #[test]
    fn parse_move() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        assert_eq!(sut.position.side_to_move, Side(Side::BLACK));
        assert_eq!(sut.position.state.castling.0, Castling::ALL);
        assert_eq!(sut.position.state.en_passant.0, Square::E3);
        assert_eq!(sut.position.half_move_number, 1);
        assert_eq!(sut.position.state.since_last_capture, 1);
        assert_eq!(
            sut.position.board.side_pieces[Side::WHITE].0,
            Square::E4 | 0b1110111111111111
        );
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            0b1111111111111111 << 48
        );
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::PAWN].0,
            Square::E4 | 0b1110111100000000
        );
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::PAWN].0,
            0b0000000011111111 << 48
        );
    }

    #[test]
    fn en_passant() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        sut.apply_algebraic_move("h7h6");
        sut.apply_algebraic_move("e4e5");
        sut.apply_algebraic_move("d7d5");
        assert_eq!(sut.position.side_to_move, Side(Side::WHITE));
        assert_eq!(sut.position.state.castling.0, Castling::ALL);
        assert_eq!(sut.position.state.en_passant.0, Square::D6);
        assert_eq!(sut.position.half_move_number, 4);
        assert_eq!(sut.position.state.since_last_capture, 4);
        assert_eq!(
            sut.position.board.side_pieces[Side::WHITE].0,
            Square::E5 | 0b1110111111111111
        );
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::PAWN].0,
            Square::E5 | 0b1110111100000000
        );
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            (Square::H6 | Square::D5 | (0b1111111101110111 << 48))
        );
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::PAWN].0,
            (Square::H6 | Square::D5 | (0b0000000001110111 << 48))
        );
        sut.apply_algebraic_move("e5d6");
        assert_eq!(sut.position.side_to_move, Side(Side::BLACK));
        assert_eq!(sut.position.state.castling.0, Castling::ALL);
        assert_eq!(sut.position.state.en_passant.0, Square::NONE);
        assert_eq!(sut.position.half_move_number, 5);
        assert_eq!(sut.position.state.since_last_capture, 0);
        assert_eq!(
            sut.position.board.side_pieces[Side::WHITE].0,
            Square::D6 | 0b1110111111111111
        );
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::PAWN].0,
            Square::D6 | 0b1110111100000000
        );
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            (Square::H6 | (0b1111111101110111 << 48))
        );
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::PAWN].0,
            (Square::H6 | (0b0000000001110111 << 48))
        );
    }

    #[test]
    fn castle() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        sut.apply_algebraic_move("e7e5");
        sut.apply_algebraic_move("g1f3");
        sut.apply_algebraic_move("g8f6");
        sut.apply_algebraic_move("f1e2");
        sut.apply_algebraic_move("f8e7");
        sut.apply_algebraic_move("e1g1");
        assert_eq!(sut.position.side_to_move, Side(Side::BLACK));
        assert_eq!(
            sut.position.state.castling.0,
            Castling::BLACK_KING_SIDE | Castling::BLACK_QUEEN_SIDE
        );
        assert_eq!(sut.position.state.en_passant.0, Square::NONE);
        assert_eq!(sut.position.half_move_number, 7);
        assert_eq!(sut.position.state.since_last_capture, 7);
        assert_eq!(
            sut.position.board.side_pieces[Side::WHITE].0,
            Square::E4 | Square::F3 | 0b1111111101101111
        );
        assert_eq!(sut.position.board.pieces[Side::WHITE][Piece::KING].0, Square::G1);
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::ROOK].0,
            Square::F1 | Square::A1
        );
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            Square::E5 | Square::F6 | (0b1001111111111111 << 48)
        );
        assert_eq!(sut.position.board.pieces[Side::BLACK][Piece::KING].0, Square::E8);
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::ROOK].0,
            Square::A8 | Square::H8
        );

        sut.apply_algebraic_move("e8g8");
        assert_eq!(sut.position.side_to_move, Side(Side::WHITE));
        assert_eq!(sut.position.state.castling.0, Castling::NO_CASTLING);
        assert_eq!(sut.position.state.en_passant.0, Square::NONE);
        assert_eq!(sut.position.half_move_number, 8);
        assert_eq!(sut.position.state.since_last_capture, 8);
        assert_eq!(
            sut.position.board.side_pieces[Side::WHITE].0,
            Square::E4 | Square::F3 | 0b1111111101101111
        );
        assert_eq!(sut.position.board.pieces[Side::WHITE][Piece::KING].0, Square::G1);
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::ROOK].0,
            Square::F1 | Square::A1
        );
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            Square::E5 | Square::F6 | (0b0110111111111111 << 48)
        );
        assert_eq!(sut.position.board.pieces[Side::BLACK][Piece::KING].0, Square::G8);
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::ROOK].0,
            Square::A8 | Square::F8
        );
    }

    #[test]
    fn capture() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("f2f4");
        sut.apply_algebraic_move("e7e5");
        sut.apply_algebraic_move("e2e4");
        sut.apply_algebraic_move("e5f4");
        assert_eq!(sut.position.side_to_move, Side(Side::WHITE));
        assert_eq!(sut.position.state.castling.0, Castling::ALL);
        assert_eq!(sut.position.state.en_passant.0, Square::NONE);
        assert_eq!(sut.position.half_move_number, 4);
        assert_eq!(sut.position.state.since_last_capture, 0);
        assert_eq!(
            sut.position.board.side_pieces[Side::WHITE].0,
            Square::E4 | 0b1100111111111111
        );
        assert_eq!(
            sut.position.board.pieces[Side::WHITE][Piece::PAWN].0,
            ((Square::SECOND_ROW | Square::E4) ^ Square::F2) ^ Square::E2
        );
        assert_eq!(
            sut.position.board.side_pieces[Side::BLACK].0,
            Square::F4 | (0b1111111111101111 << 48)
        );
        assert_eq!(
            sut.position.board.pieces[Side::BLACK][Piece::PAWN].0,
            (Square::SEVENTH_ROW | Square::F4) ^ Square::E7
        );
    }
}

#[cfg(test)]
mod position_tests {
    use crate::board::models::{Castling, Position, Side, Square};

    #[test]
    fn empty_pos() {
        let sut = Position::empty();
        assert_eq!(sut.side_to_move, Side(Side::WHITE));
        assert_eq!(sut.state.castling.0, Castling::NO_CASTLING);
        assert_eq!(sut.state.en_passant.0, Square::NONE);
        assert_eq!(sut.half_move_number, 0);
        assert_eq!(sut.state.since_last_capture, 0);
    }
}

#[cfg(test)]
mod utils_tests {
    use crate::board::{
        models::Square,
        utils::{algebraic_to_square, square_to_algebraic},
    };

    #[test]
    fn square_to_alg() {
        assert_eq!("a1", square_to_algebraic(Square::A1));
        assert_eq!("a8", square_to_algebraic(Square::A8));
        assert_eq!("e2", square_to_algebraic(Square::E2));
        assert_eq!("h1", square_to_algebraic(Square::H1));
        assert_eq!("h8", square_to_algebraic(Square::H8));
        assert_eq!("g3", square_to_algebraic(Square::G3));
        assert_eq!("e7", square_to_algebraic(Square::E7));
    }

    #[test]
    fn alg_to_square() {
        assert_eq!(Square::A1, algebraic_to_square("a1"));
        assert_eq!(Square::A8, algebraic_to_square("a8"));
        assert_eq!(Square::E2, algebraic_to_square("e2"));
        assert_eq!(Square::H1, algebraic_to_square("h1"));
        assert_eq!(Square::H8, algebraic_to_square("h8"));
        assert_eq!(Square::G3, algebraic_to_square("g3"));
        assert_eq!(Square::E7, algebraic_to_square("e7"));
    }
}

#[cfg(test)]
mod zobrist_tests {
    use crate::{board::{fen::FenParser, models::{Engine, Square, Move, Piece, Castling}}, constants::START_POS, movegen::generator::MoveInfo};

    #[test]
    fn works_and_is_same() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        let hash1 = sut.position.zobrist.clone();
        sut.apply_algebraic_move("e7e5");
        sut.apply_algebraic_move("e4e2");
        sut.apply_algebraic_move("e5e7");
        sut.apply_algebraic_move("e2e4");
        assert_eq!(hash1.hash, sut.position.zobrist.hash);
    }

    #[test]
    fn undo_works() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        let m = MoveInfo {
            m: Move::Normal(Square::E2, Square::E4),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m);
        let hash1 = sut.position.zobrist.clone();
        let m = MoveInfo {
            m: Move::Normal(Square::E7, Square::E5),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m);
        sut.undo_move(&m);
        assert_eq!(hash1.hash, sut.position.zobrist.hash);
    }

    #[test]
    fn double_undo_works() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        let hash1 = sut.position.zobrist.clone();
        let m = MoveInfo {
            m: Move::Normal(Square::E7, Square::E5),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m);
        let m2 = MoveInfo {
            m: Move::Normal(Square::F2, Square::F4),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m2);
        sut.undo_move(&m2);
        sut.undo_move(&m);
        assert_eq!(hash1.hash, sut.position.zobrist.hash);
    }

    #[test]
    fn a_lot_of_undos_work() {
        let fen = START_POS;
        let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
        sut.apply_algebraic_move("e2e4");
        let hash1 = sut.position.zobrist.clone();
        let m = MoveInfo {
            m: Move::Normal(Square::E7, Square::E5),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m);
        let m1 = MoveInfo {
            m: Move::Normal(Square::F2, Square::F4),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m1);
        let m2 = MoveInfo {
            m: Move::Normal(Square::F7, Square::F5),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m2);
        let m3 = MoveInfo {
            m: Move::Normal(Square::G1, Square::F3),
            piece: Piece::KNIGHT,
            captured_piece: None,
        };
        sut.apply_move(&m3);
        let m4 = MoveInfo {
            m: Move::Normal(Square::G8, Square::F6),
            piece: Piece::KNIGHT,
            captured_piece: None,
        };
        sut.apply_move(&m4);
        let m5 = MoveInfo {
            m: Move::Normal(Square::F1, Square::E2),
            piece: Piece::BISHOP,
            captured_piece: None,
        };
        sut.apply_move(&m5);
        let m6 = MoveInfo {
            m: Move::Normal(Square::F8, Square::E7),
            piece: Piece::BISHOP,
            captured_piece: None,
        };
        sut.apply_move(&m6);
        let m7 = MoveInfo {
            m: Move::Castle(Castling::WHITE_KING_SIDE),
            piece: Piece::KING,
            captured_piece: None,
        };
        sut.apply_move(&m7);
        let m8 = MoveInfo {
            m: Move::Castle(Castling::BLACK_KING_SIDE),
            piece: Piece::KING,
            captured_piece: None,
        };
        sut.apply_move(&m8);
        let m9 = MoveInfo {
            m: Move::Normal(Square::A2, Square::A4),
            piece: Piece::PAWN,
            captured_piece: None,
        };
        sut.apply_move(&m9);
        sut.undo_move(&m);
        sut.undo_move(&m1);
        sut.undo_move(&m2);
        sut.undo_move(&m3);
        sut.undo_move(&m4);
        sut.undo_move(&m5);
        sut.undo_move(&m6);
        sut.undo_move(&m7);
        sut.undo_move(&m8);
        sut.undo_move(&m9);
        assert_eq!(hash1.hash, sut.position.zobrist.hash);
    }
}
