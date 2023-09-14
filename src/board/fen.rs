use std::sync::Arc;

use super::{
    models::{BoardState, Castling, Piece, Position, Side, Square},
    utils::algebraic_to_square,
};

pub struct FenParser;

impl FenParser {
    pub fn fen_to_position(fen: &str) -> Position {
        let mut position = Position::empty();
        let mut fen = fen.split(' ');

        // Pieces position
        Self::parse_piece_position(&mut fen, &mut position);

        // Side to move
        for c in fen.next().unwrap().chars() {
            position.side_to_move = if c == 'w' {
                Side(Side::WHITE)
            } else {
                Side(Side::BLACK)
            }
        }

        let mut castling = Castling(Castling::NO_CASTLING);
        // Castling
        for c in fen.next().unwrap().chars() {
            match c {
                'Q' => castling.0 |= Castling::WHITE_QUEEN_SIDE,
                'K' => castling.0 |= Castling::WHITE_KING_SIDE,
                'q' => castling.0 |= Castling::BLACK_QUEEN_SIDE,
                'k' => castling.0 |= Castling::BLACK_KING_SIDE,
                _ => {}
            }
        }

        let mut en_passant = Square(Square::NONE);
        // En passant
        let square = fen.next().unwrap();
        if square != "-" {
            en_passant = Square(algebraic_to_square(square));
        }

        // Moves since last capture
        let since_last_capture: usize = fen.next().unwrap().parse().unwrap();

        // Assign state
        position.state = Arc::new(BoardState {
            castling,
            en_passant,
            since_last_capture,
            since_last_capture_or_pawn_movement: since_last_capture,
            prev: None
        });

        // Half moves
        let full_moves: usize = fen.next().unwrap().parse().unwrap();
        position.half_move_number = (full_moves - 1) * 2;
        if position.side_to_move == Side(Side::BLACK) {
            position.half_move_number += 1;
        }
        position.zobrist = Arc::new(position.init_zobrist_key());

        position
    }

    // Parse the piece position from the fen string
    fn parse_piece_position(fen: &mut std::str::Split<char>, position: &mut Position) {
        let mut i: i32 = 64 - 8;
        for c in fen.next().unwrap().chars() {
            match c {
                '/' => i -= 16,
                'r' => {
                    position.set_piece(1u64 << i, Side::BLACK, Piece::ROOK);
                    i += 1
                }
                'R' => {
                    position.set_piece(1u64 << i, Side::WHITE, Piece::ROOK);
                    i += 1
                }
                'b' => {
                    position.set_piece(1u64 << i, Side::BLACK, Piece::BISHOP);
                    i += 1
                }
                'B' => {
                    position.set_piece(1u64 << i, Side::WHITE, Piece::BISHOP);
                    i += 1
                }
                'n' => {
                    position.set_piece(1u64 << i, Side::BLACK, Piece::KNIGHT);
                    i += 1
                }
                'N' => {
                    position.set_piece(1u64 << i, Side::WHITE, Piece::KNIGHT);
                    i += 1
                }
                'q' => {
                    position.set_piece(1u64 << i, Side::BLACK, Piece::QUEEN);
                    i += 1
                }
                'Q' => {
                    position.set_piece(1u64 << i, Side::WHITE, Piece::QUEEN);
                    i += 1
                }
                'k' => {
                    position.set_piece(1u64 << i, Side::BLACK, Piece::KING);
                    i += 1
                }
                'K' => {
                    position.set_piece(1u64 << i, Side::WHITE, Piece::KING);
                    i += 1
                }
                'p' => {
                    position.set_piece(1u64 << i, Side::BLACK, Piece::PAWN);
                    i += 1
                }
                'P' => {
                    position.set_piece(1u64 << i, Side::WHITE, Piece::PAWN);
                    i += 1
                }
                other => match other.to_digit(10) {
                    Some(n) => {
                        i += n as i32;
                    }
                    None => { /* Ignore */ }
                },
            }
        }
    }
}
