use std::{sync::Arc, time::Instant};

use rustc_hash::FxHashMap;

use crate::{
    constants::{BISHOP_VALUE, KING_VALUE, KNIGHT_VALUE, PAWN_VALUE, QUEEN_VALUE, ROOK_VALUE},
    evaluate::evaluator::Evaluate,
    movegen::generator::MoveInfo,
};

use super::{
    utils::algebraic_to_move,
    zobrist::{ZobristHashes, ZobristValue},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Engine {
    pub position: Position,
    pub current_best_move: Option<MoveInfo>,
    pub hash_history: Vec<u64>,
    pub is_searching: bool,
    pub is_configuring: bool,
    pub zobrist_table: FxHashMap<u64, [Option<(isize, isize, isize)>; 10]>,
    pub zobrist_evaluation_table: FxHashMap<u64, isize>,
    pub searched_nodes: u128,
    pub started_searching_at: Option<Instant>,
    pub static_eval: Option<isize>,
}
impl Engine {
    pub fn empty() -> Engine {
        Engine {
            position: Position::empty(),
            current_best_move: None,
            hash_history: Vec::new(),
            is_searching: false,
            is_configuring: false,
            zobrist_table: FxHashMap::default(),
            zobrist_evaluation_table: FxHashMap::default(),
            searched_nodes: 0,
            started_searching_at: None,
            static_eval: None,
        }
    }

    pub fn from_position(position: Position) -> Engine {
        Engine {
            position,
            current_best_move: None,
            hash_history: Vec::new(),
            is_searching: false,
            is_configuring: false,
            zobrist_table: FxHashMap::default(),
            zobrist_evaluation_table: FxHashMap::default(),
            searched_nodes: 0,
            started_searching_at: None,
            static_eval: None,
        }
    }

    pub fn apply_move(&mut self, m: &MoveInfo) {
        if self.static_eval.is_some() {
            self.update_static_eval(m);
        } else {
            self.static_eval = Some(Evaluate::evaluate_material(&self));
        }
        self.position.apply_move(m);
        self.hash_history.push(self.position.zobrist.hash);
    }

    pub fn undo_move(&mut self, m: &MoveInfo) {
        self.update_static_eval(m);
        self.position.undo_move(m);
        self.hash_history.pop();
    }

    fn update_static_eval(&mut self, m: &MoveInfo) {
        if let Some(captured_piece) = m.captured_piece {
            let piece_value = match captured_piece {
                Piece::PAWN => PAWN_VALUE,
                Piece::KNIGHT => KNIGHT_VALUE,
                Piece::BISHOP => BISHOP_VALUE,
                Piece::ROOK => ROOK_VALUE,
                Piece::QUEEN => QUEEN_VALUE,
                Piece::KING => KING_VALUE,
                _ => 0,
            };
            self.static_eval = Some(
                self.static_eval.unwrap()
                    + (match self.position.side_to_move {
                        Side(Side::WHITE) => piece_value,
                        Side(Side::BLACK) => -piece_value,
                        _ => 0,
                    }),
            );
        }
        if let Move::Promotion(_, _, promotion_piece) = m.m {
            let promotion_piece_value = (match promotion_piece {
                Piece::KNIGHT => KNIGHT_VALUE,
                Piece::BISHOP => BISHOP_VALUE,
                Piece::ROOK => ROOK_VALUE,
                Piece::QUEEN => QUEEN_VALUE,
                _ => 0,
            }) - PAWN_VALUE;
            self.static_eval = Some(
                self.static_eval.unwrap()
                    + (match self.position.side_to_move {
                        Side(Side::WHITE) => promotion_piece_value,
                        Side(Side::BLACK) => -promotion_piece_value,
                        _ => 0,
                    }),
            );
        }
    }

    // Apply a move given as a string in long algebraic notation
    pub fn apply_algebraic_move(&mut self, alg_move: &str) {
        let actual_move = algebraic_to_move(alg_move, &self.position);
        let from_square = match actual_move {
            Move::Normal(from, _) => from,
            Move::Castle(castling) => match castling {
                Castling::WHITE_KING_SIDE => Square::E1,
                Castling::WHITE_QUEEN_SIDE => Square::E1,
                Castling::BLACK_KING_SIDE => Square::E8,
                Castling::BLACK_QUEEN_SIDE => Square::E8,
                _ => Square::NONE,
            },
            Move::Promotion(from, _, _) => from,
            Move::EnPassant(from, _) => from,
        };
        let to_square = match actual_move {
            Move::Normal(_, to) => Some(to),
            Move::Castle(_) => None,
            Move::Promotion(_, to, _) => Some(to),
            Move::EnPassant(_, to) => match self.position.side_to_move.0 {
                Side::WHITE => Some(to >> 8),
                Side::BLACK => Some(to << 8),
                _ => None,
            },
        };
        // Find out which piece we are moving and capturing
        let mut piece = 0;
        let mut captured_piece: Option<usize> = None;
        for (index, piece_board) in self.position.board.pieces[self.position.side_to_move.0]
            .iter()
            .enumerate()
        {
            if (piece_board.0 & from_square) != 0 {
                // Got it! Move this piece
                piece = index;
                break;
            }
        }
        if let Some(to_square) = to_square {
            for (index, piece_board) in self.position.board.pieces[self.position.opposite_side()]
                .iter()
                .enumerate()
            {
                if (piece_board.0 & to_square) != 0 {
                    // Got it! Move this piece
                    captured_piece = Some(index);
                    break;
                }
            }
        }
        self.position.apply_move(&MoveInfo {
            m: actual_move,
            piece,
            captured_piece: if captured_piece.is_none() {
                None
            } else {
                captured_piece
            },
        });
        self.hash_history.push(self.position.zobrist.hash);
    }

    pub fn check_draw_coditions(&mut self) -> bool {
        // Fifty-move rule
        if self.position.state.since_last_capture_or_pawn_movement > 50 {
            return true;
        }

        // Threefold repetition
        if self.position.state.since_last_capture_or_pawn_movement >= 8
            && self.hash_history.len() > 0
        {
            // We need at least 9 half moves
            let hash_len = self.hash_history.len();
            let this_position = self.hash_history[hash_len - 1];
            // It's impossibile to repeat a state after a capture or pawn movement
            let positions_to_check = self
                .hash_history
                .iter()
                .rev()
                .take(self.position.state.since_last_capture_or_pawn_movement + 1);
            let mut times_repeated = 0;
            for position in positions_to_check {
                if this_position == *position {
                    times_repeated += 1;
                }
                if times_repeated >= 3 {
                    // Draws!
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Move {
    Normal(u64, u64),
    Castle(u8),
    EnPassant(u64, u64),
    Promotion(u64, u64, usize),
}

// The current state of the game
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Position {
    pub board: PiecePosition,
    pub side_to_move: Side,
    pub half_move_number: usize,
    pub state: Arc<BoardState>,
    pub zobrist: Arc<ZobristValue>,
    pub zobrist_hashes: ZobristHashes,
}
impl Position {
    pub fn empty() -> Position {
        Position {
            board: PiecePosition::empty(),
            side_to_move: Side(Side::WHITE),
            half_move_number: 0,
            state: Arc::new(BoardState::empty()),
            zobrist_hashes: ZobristHashes::init(),
            zobrist: Arc::new(ZobristValue::empty()),
        }
    }

    // Get the side that is not moving in this turn
    pub fn opposite_side(&self) -> usize {
        match self.side_to_move.0 {
            Side::BLACK => Side::WHITE,
            Side::WHITE => Side::BLACK,
            _ => 0,
        }
    }

    // Set a piece, dumb way to set a square to a piece, only used while parsing FENs
    pub fn set_piece(&mut self, pos: u64, side: usize, piece: usize) {
        self.board.side_pieces[side].0 |= pos;
        self.board.pieces[side][piece].0 |= pos;
    }

    // Rapidly undos the last move that was made by the engine
    pub fn undo_move(&mut self, m: &MoveInfo) {
        self.side_to_move = Side(self.opposite_side());
        let is_capture = m.captured_piece.is_some();
        match m.m {
            // Normal move
            Move::Normal(from, to) => {
                // Change back side reprs
                self.set_side_pieces_back_from_move(to, from, is_capture);
                // Move the piece back
                self.board.pieces[self.side_to_move.0][m.piece].0 &= !to;
                self.board.pieces[self.side_to_move.0][m.piece].0 |= from;
                if let Some(captured) = m.captured_piece {
                    // Place back the captured piece
                    self.board.pieces[self.opposite_side()][captured].0 |= to;
                }
            }
            // Promotion
            Move::Promotion(from, to, piece) => {
                // Change back side reprs
                self.set_side_pieces_back_from_move(to, from, is_capture);
                // Move the piece back
                self.board.pieces[self.side_to_move.0][piece].0 &= !to;
                self.board.pieces[self.side_to_move.0][Piece::PAWN].0 |= from;
                if let Some(captured) = m.captured_piece {
                    // Place back the captured piece
                    self.board.pieces[self.opposite_side()][captured].0 |= to;
                }
            }
            // En passant
            Move::EnPassant(from, to) => {
                // Add back the captured pawn
                let square_to_populate = if self.side_to_move.0 == Side::WHITE {
                    to.wrapping_shr(8)
                } else {
                    to.wrapping_shl(8)
                };
                // Change back side reprs
                self.set_side_pieces_back_from_enpassant(to, from, square_to_populate);
                // Move the piece back
                self.board.pieces[self.side_to_move.0][Piece::PAWN].0 &= !to;
                self.board.pieces[self.side_to_move.0][Piece::PAWN].0 |= from;
                self.board.pieces[self.opposite_side()][Piece::PAWN].0 |= square_to_populate;
            }
            // Castle
            Move::Castle(castling) => match castling {
                Castling::WHITE_KING_SIDE => {
                    self.set_side_pieces_back_from_move(Square::G1, Square::E1, false);
                    self.set_side_pieces_back_from_move(Square::F1, Square::H1, false);
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 &= !Square::G1;
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 |= Square::E1;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 &= !Square::F1;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 |= Square::H1;
                }
                Castling::WHITE_QUEEN_SIDE => {
                    self.set_side_pieces_back_from_move(Square::C1, Square::E1, false);
                    self.set_side_pieces_back_from_move(Square::D1, Square::A1, false);
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 &= !Square::C1;
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 |= Square::E1;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 &= !Square::D1;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 |= Square::A1;
                }
                Castling::BLACK_KING_SIDE => {
                    self.set_side_pieces_back_from_move(Square::G8, Square::E8, false);
                    self.set_side_pieces_back_from_move(Square::F8, Square::H8, false);
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 &= !Square::G8;
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 |= Square::E8;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 &= !Square::F8;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 |= Square::H8;
                }
                Castling::BLACK_QUEEN_SIDE => {
                    self.set_side_pieces_back_from_move(Square::C8, Square::E8, false);
                    self.set_side_pieces_back_from_move(Square::D8, Square::A8, false);
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 &= !Square::C8;
                    self.board.pieces[self.side_to_move.0][Piece::KING].0 |= Square::E8;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 &= !Square::D8;
                    self.board.pieces[self.side_to_move.0][Piece::ROOK].0 |= Square::A8;
                }
                _ => {}
            },
        }
        self.zobrist = Arc::clone(&self.zobrist.prev.as_ref().unwrap());
        self.state = Arc::clone(&self.state.prev.as_ref().unwrap());
        self.half_move_number -= 1;
    }

    // Actually apply a move
    pub fn apply_move(&mut self, move_action: &MoveInfo) {
        let mut new_zobrist = ZobristValue {
            hash: self.zobrist.hash,
            prev: Some(Arc::clone(&self.zobrist)),
        };
        let mut new_state = BoardState {
            castling: Castling(self.state.castling.0),
            en_passant: Square(self.state.en_passant.0),
            since_last_capture: self.state.since_last_capture,
            since_last_capture_or_pawn_movement: self.state.since_last_capture_or_pawn_movement,
            prev: Some(Arc::clone(&self.state)),
        };

        let own_rooks_square = self.board.pieces[self.side_to_move.0][Piece::ROOK].0;
        let opposite_rooks_square = self.board.pieces[self.opposite_side()][Piece::ROOK].0;

        let has_king_moved = move_action.piece == Piece::KING;
        let is_capture = move_action.captured_piece.is_some();
        let mut has_queen_rook_moved = false;
        let mut has_king_rook_moved = false;
        let mut has_opposite_side_queen_rook_been_captured = false;
        let mut has_opposite_side_king_rook_been_captured = false;

        let mut enpassant_square = Square::NONE;

        match move_action.m {
            // Normal move
            Move::Normal(from, to) => {
                // Check if we are moving a pawn 2 times
                if move_action.piece == Piece::PAWN && self.check_is_pawn_moving_2_squares(from, to)
                {
                    enpassant_square = to;
                }

                // Update side reprs
                self.set_side_pieces_from_move(from, to, is_capture);

                // Move the pieces
                self.move_piece_from_move(from, to, move_action, &mut new_zobrist);

                if !has_king_moved && move_action.piece == Piece::ROOK {
                    let queen_rook_square = if self.side_to_move.0 == Side::WHITE {
                        Square::A1
                    } else {
                        Square::A8
                    };
                    let king_rook_square = if self.side_to_move.0 == Side::WHITE {
                        Square::H1
                    } else {
                        Square::H8
                    };

                    has_queen_rook_moved = (queen_rook_square & own_rooks_square & from) > 0;

                    has_king_rook_moved = (king_rook_square & own_rooks_square & from) > 0;
                }
                if is_capture && move_action.captured_piece.unwrap() == Piece::ROOK {
                    let opposite_queen_rook_square = if self.side_to_move.0 == Side::WHITE {
                        Square::A8
                    } else {
                        Square::A1
                    };
                    let opposite_king_rook_square = if self.side_to_move.0 == Side::WHITE {
                        Square::H8
                    } else {
                        Square::H1
                    };
                    has_opposite_side_queen_rook_been_captured =
                        (opposite_queen_rook_square & opposite_rooks_square & to) > 0;
                    has_opposite_side_king_rook_been_captured =
                        (opposite_king_rook_square & opposite_rooks_square & to) > 0;
                }
            }
            // Promotion
            Move::Promotion(from, to, piece) => {
                // Update side reprs
                self.set_side_pieces_from_move(from, to, is_capture);

                self.promote_from_move(
                    from,
                    to,
                    piece,
                    move_action.captured_piece,
                    &mut new_zobrist,
                );

                if is_capture && move_action.captured_piece.unwrap() == Piece::ROOK {
                    let opposite_queen_rook_square = if self.side_to_move.0 == Side::WHITE {
                        Square::A8
                    } else {
                        Square::A1
                    };
                    let opposite_king_rook_square = if self.side_to_move.0 == Side::WHITE {
                        Square::H8
                    } else {
                        Square::H1
                    };
                    has_opposite_side_queen_rook_been_captured =
                        (opposite_queen_rook_square & opposite_rooks_square & to) > 0;
                    has_opposite_side_king_rook_been_captured =
                        (opposite_king_rook_square & opposite_rooks_square & to) > 0;
                }
            }
            // En passant
            Move::EnPassant(from, to) => {
                // Remove the captured pawn
                let square_to_capture = if self.side_to_move.0 == Side::WHITE {
                    to.wrapping_shr(8)
                } else {
                    to.wrapping_shl(8)
                };
                // Update side reprs
                self.set_side_pieces_from_enpassant(from, to, square_to_capture);

                // Move the pawn
                self.remove_piece(self.side_to_move.0, Piece::PAWN, from, &mut new_zobrist);
                self.add_piece(self.side_to_move.0, Piece::PAWN, to, &mut new_zobrist);

                self.remove_piece(
                    self.opposite_side(),
                    Piece::PAWN,
                    square_to_capture,
                    &mut new_zobrist,
                );
            }
            // Castle
            Move::Castle(castling) => match castling {
                Castling::WHITE_KING_SIDE => {
                    has_king_rook_moved = true;
                    self.castle(
                        Square::E1,
                        Square::G1,
                        Square::H1,
                        Square::F1,
                        &mut new_zobrist,
                    );
                }
                Castling::WHITE_QUEEN_SIDE => {
                    has_queen_rook_moved = true;
                    self.castle(
                        Square::E1,
                        Square::C1,
                        Square::A1,
                        Square::D1,
                        &mut new_zobrist,
                    );
                }
                Castling::BLACK_KING_SIDE => {
                    has_king_rook_moved = true;
                    self.castle(
                        Square::E8,
                        Square::G8,
                        Square::H8,
                        Square::F8,
                        &mut new_zobrist,
                    );
                }
                Castling::BLACK_QUEEN_SIDE => {
                    has_queen_rook_moved = true;
                    self.castle(
                        Square::E8,
                        Square::C8,
                        Square::A8,
                        Square::D8,
                        &mut new_zobrist,
                    );
                }
                _ => {}
            },
        }

        // Add to number of moves
        self.half_move_number += 1;

        // ** Modify the state **

        // Add half moves since capture
        new_state.since_last_capture = if is_capture {
            0
        } else {
            new_state.since_last_capture + 1
        };
        // Add half moves since capture or pawn movement
        new_state.since_last_capture_or_pawn_movement =
            if is_capture || move_action.piece == Piece::PAWN {
                0
            } else {
                new_state.since_last_capture_or_pawn_movement + 1
            };

        // Check en passant square
        if enpassant_square != Square::NONE {
            if self.side_to_move == Side(Side::WHITE) {
                new_state.en_passant.0 = enpassant_square >> 8
            } else {
                new_state.en_passant.0 = enpassant_square << 8
            }
        } else {
            new_state.en_passant.0 = Square::NONE;
        }
        let prev_enpassant = new_state.prev.as_ref().unwrap().en_passant.0;
        if new_state.en_passant.0 != prev_enpassant {
            new_zobrist.hash ^= self
                .zobrist_hashes
                .en_passant(prev_enpassant.trailing_zeros() as usize)
                .hash;
            new_zobrist.hash ^= self
                .zobrist_hashes
                .en_passant(new_state.en_passant.0.trailing_zeros() as usize)
                .hash;
        }

        // Check castling
        if new_state.castling.0 != Castling::NO_CASTLING {
            if has_king_moved {
                if self.side_to_move == Side(Side::WHITE) {
                    new_state.castling.0 &= !Castling(Castling::WHITE_KING_SIDE).0;
                    new_state.castling.0 &= !Castling(Castling::WHITE_QUEEN_SIDE).0;
                } else {
                    new_state.castling.0 &= !Castling(Castling::BLACK_KING_SIDE).0;
                    new_state.castling.0 &= !Castling(Castling::BLACK_QUEEN_SIDE).0;
                }
            } else {
                if has_king_rook_moved {
                    if self.side_to_move == Side(Side::WHITE) {
                        new_state.castling.0 &= !Castling(Castling::WHITE_KING_SIDE).0;
                    } else {
                        new_state.castling.0 &= !Castling(Castling::BLACK_KING_SIDE).0;
                    }
                }
                if has_queen_rook_moved {
                    if self.side_to_move == Side(Side::WHITE) {
                        new_state.castling.0 &= !Castling(Castling::WHITE_QUEEN_SIDE).0;
                    } else {
                        new_state.castling.0 &= !Castling(Castling::BLACK_QUEEN_SIDE).0;
                    }
                }
            }
            if has_opposite_side_queen_rook_been_captured {
                if self.side_to_move == Side(Side::WHITE) {
                    new_state.castling.0 &= !Castling(Castling::BLACK_QUEEN_SIDE).0;
                } else {
                    new_state.castling.0 &= !Castling(Castling::WHITE_QUEEN_SIDE).0;
                }
            }
            if has_opposite_side_king_rook_been_captured {
                if self.side_to_move == Side(Side::WHITE) {
                    new_state.castling.0 &= !Castling(Castling::BLACK_KING_SIDE).0;
                } else {
                    new_state.castling.0 &= !Castling(Castling::WHITE_KING_SIDE).0;
                }
            }
            if has_king_moved
                || has_king_rook_moved
                || has_queen_rook_moved
                || has_opposite_side_king_rook_been_captured
                || has_opposite_side_queen_rook_been_captured
            {
                let prev_castling = new_state.prev.as_ref().unwrap().castling.0;
                if new_state.castling.0 != prev_castling {
                    new_zobrist.hash ^= self.zobrist_hashes.castling(prev_castling as usize).hash;
                    new_zobrist.hash ^= self
                        .zobrist_hashes
                        .castling(new_state.castling.0 as usize)
                        .hash;
                }
            }
        }

        // Change moving side
        self.side_to_move = if self.side_to_move == Side(Side::WHITE) {
            Side(Side::BLACK)
        } else {
            Side(Side::WHITE)
        };
        new_zobrist.hash ^= self.zobrist_hashes.side(self.side_to_move.0).hash;

        self.zobrist = Arc::new(new_zobrist);
        self.state = Arc::new(new_state);
    }

    fn remove_piece(
        &mut self,
        side: usize,
        piece: usize,
        from: u64,
        new_zobrist: &mut ZobristValue,
    ) {
        let square = from.trailing_zeros();
        new_zobrist.hash ^= self.zobrist_hashes.piece(side, piece, square as usize).hash;
        self.board.pieces[side][piece].0 &= !from;
    }

    fn add_piece(&mut self, side: usize, piece: usize, to: u64, new_zobrist: &mut ZobristValue) {
        let square = to.trailing_zeros();
        new_zobrist.hash ^= self.zobrist_hashes.piece(side, piece, square as usize).hash;
        self.board.pieces[side][piece].0 |= to;
    }

    fn castle(
        &mut self,
        from: u64,
        to: u64,
        from_rook: u64,
        to_rook: u64,
        new_zobrist: &mut ZobristValue,
    ) {
        // King
        // Update side reprs
        self.set_side_pieces_from_move(from, to, false);
        // Move the king
        self.remove_piece(self.side_to_move.0, Piece::KING, from, new_zobrist);
        self.add_piece(self.side_to_move.0, Piece::KING, to, new_zobrist);
        // Rook
        // Update side reprs
        self.set_side_pieces_from_move(from_rook, to_rook, false);
        // Move the rook
        self.remove_piece(self.side_to_move.0, Piece::ROOK, from_rook, new_zobrist);
        self.add_piece(self.side_to_move.0, Piece::ROOK, to_rook, new_zobrist);
    }

    fn move_piece_from_move(
        &mut self,
        from: u64,
        to: u64,
        move_action: &MoveInfo,
        new_zobrist: &mut ZobristValue,
    ) {
        self.remove_piece(self.side_to_move.0, move_action.piece, from, new_zobrist);
        self.add_piece(self.side_to_move.0, move_action.piece, to, new_zobrist);

        // Eventually capture
        if let Some(captured_piece) = move_action.captured_piece {
            self.remove_piece(self.opposite_side(), captured_piece, to, new_zobrist);
        }
    }

    fn promote_from_move(
        &mut self,
        from: u64,
        to: u64,
        piece: usize,
        captured_piece: Option<usize>,
        new_zobrist: &mut ZobristValue,
    ) {
        self.remove_piece(self.side_to_move.0, Piece::PAWN, from, new_zobrist);
        self.add_piece(self.side_to_move.0, piece, to, new_zobrist);

        // Capture if needed
        if let Some(captured_piece) = captured_piece {
            self.remove_piece(self.opposite_side(), captured_piece, to, new_zobrist);
        }
    }

    fn set_side_pieces_from_move(&mut self, from: u64, to: u64, is_capture: bool) {
        self.board.side_pieces[self.side_to_move.0].0 &= !from;
        if is_capture {
            self.board.side_pieces[self.opposite_side()].0 &= !to;
        }
        self.board.side_pieces[self.side_to_move.0].0 |= to;
    }

    fn set_side_pieces_back_from_move(&mut self, to: u64, from: u64, is_capture: bool) {
        self.board.side_pieces[self.side_to_move.0].0 &= !to;
        if is_capture {
            self.board.side_pieces[self.opposite_side()].0 |= to;
        }
        self.board.side_pieces[self.side_to_move.0].0 |= from;
    }

    fn set_side_pieces_from_enpassant(&mut self, from: u64, to: u64, square_to_capture: u64) {
        self.board.side_pieces[self.side_to_move.0].0 &= !from;
        self.board.side_pieces[self.opposite_side()].0 &= !square_to_capture;
        self.board.side_pieces[self.side_to_move.0].0 |= to;
    }

    fn set_side_pieces_back_from_enpassant(&mut self, to: u64, from: u64, square_to_populate: u64) {
        self.board.side_pieces[self.side_to_move.0].0 &= !to;
        self.board.side_pieces[self.opposite_side()].0 |= square_to_populate;
        self.board.side_pieces[self.side_to_move.0].0 |= from;
    }

    fn check_is_pawn_moving_2_squares(&self, from: u64, to: u64) -> bool {
        ((self.board.pieces[self.side_to_move.0][Piece::PAWN].0 & from) != 0)
            && ((from == (to >> 16)) || (from == (to << 16)))
    }

    // First init of the hash
    // Only needed at the start, since then we can just update it incrementally
    // (Thanks rustic for the algorithm)
    pub fn init_zobrist_key(&self) -> ZobristValue {
        let mut result: u64 = 0;
        let white_pieces = self.board.pieces[Side::WHITE];
        let black_pieces = self.board.pieces[Side::BLACK];

        for (piece_type, (w, b)) in white_pieces.iter().zip(black_pieces.iter()).enumerate() {
            let mut white_pieces = *w;
            let mut black_pieces = *b;

            while white_pieces.0 > 0 {
                let square = white_pieces.0.trailing_zeros();
                white_pieces.0 ^= 1u64 << square;
                result ^= self
                    .zobrist_hashes
                    .piece(Side::WHITE, piece_type, square as usize)
                    .hash;
            }

            while black_pieces.0 > 0 {
                let square = black_pieces.0.trailing_zeros();
                black_pieces.0 ^= 1u64 << square;
                result ^= self
                    .zobrist_hashes
                    .piece(Side::BLACK, piece_type, square as usize)
                    .hash;
            }
        }

        result ^= self
            .zobrist_hashes
            .castling(self.state.castling.0 as usize)
            .hash;
        result ^= self.zobrist_hashes.side(self.side_to_move.0).hash;
        result ^= self
            .zobrist_hashes
            .en_passant(self.state.en_passant.0.trailing_zeros() as usize)
            .hash;

        ZobristValue {
            hash: result,
            prev: None,
        }
    }
}

// The current (not pieces) state of the board
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct BoardState {
    pub since_last_capture: usize,
    pub since_last_capture_or_pawn_movement: usize,
    pub castling: Castling,
    pub en_passant: Square,
    pub prev: Option<Arc<BoardState>>,
}
impl BoardState {
    pub fn empty() -> BoardState {
        BoardState {
            castling: Castling(Castling::NO_CASTLING),
            en_passant: Square(Square::NONE),
            since_last_capture: 0,
            since_last_capture_or_pawn_movement: 0,
            prev: None,
        }
    }
}

// The board as a 64bit integer
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Default, Hash)]
pub struct BitBoard(pub u64);

// The position of all the pieces in the board
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PiecePosition {
    // All the pieces
    pub side_pieces: [BitBoard; 2],
    // Single pieces
    pub pieces: [[BitBoard; 6]; 2],
}
impl PiecePosition {
    pub fn empty() -> PiecePosition {
        PiecePosition {
            side_pieces: [BitBoard(0); 2],
            pieces: [[BitBoard(0); 6]; 2],
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Side(pub usize);
impl Side {
    pub const WHITE: usize = 0;
    pub const BLACK: usize = 1;
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Piece(pub usize);
impl Piece {
    pub const PAWN: usize = 0;
    pub const BISHOP: usize = 1;
    pub const KNIGHT: usize = 2;
    pub const ROOK: usize = 3;
    pub const QUEEN: usize = 4;
    pub const KING: usize = 5;
}

// The castling rights as a 8 bit value, the last 4 bit only have a value
// 00001010
// (nothing, nothing, nothing, nothing, white queen side, white king side, black queen side, white queen side)
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Castling(pub u8);
impl Castling {
    pub const NO_CASTLING: u8 = 0b00000000;
    pub const WHITE_QUEEN_SIDE: u8 = 0b00001000;
    pub const WHITE_KING_SIDE: u8 = 0b00000100;
    pub const BLACK_QUEEN_SIDE: u8 = 0b00000010;
    pub const BLACK_KING_SIDE: u8 = 0b00000001;
    #[allow(dead_code)]
    pub const ALL_WHITE: u8 = Self::WHITE_QUEEN_SIDE | Self::WHITE_KING_SIDE;
    #[allow(dead_code)]
    pub const ALL_BLACK: u8 = Self::BLACK_QUEEN_SIDE | Self::BLACK_KING_SIDE;
    #[allow(dead_code)]
    pub const ALL: u8 = Self::ALL_WHITE | Self::ALL_BLACK;
}

// The squares of the board, used for en-passant
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Square(pub u64);
#[allow(dead_code)]
impl Square {
    pub const NONE: u64 = 0;
    pub const A1: u64 = 0b1u64;
    pub const B1: u64 = 0b1u64 << 1;
    pub const C1: u64 = 0b1u64 << 2;
    pub const D1: u64 = 0b1u64 << 3;
    pub const E1: u64 = 0b1u64 << 4;
    pub const F1: u64 = 0b1u64 << 5;
    pub const G1: u64 = 0b1u64 << 6;
    pub const H1: u64 = 0b1u64 << 7;
    pub const A2: u64 = 0b1u64 << 8;
    pub const B2: u64 = 0b1u64 << 9;
    pub const C2: u64 = 0b1u64 << 10;
    pub const D2: u64 = 0b1u64 << 11;
    pub const E2: u64 = 0b1u64 << 12;
    pub const F2: u64 = 0b1u64 << 13;
    pub const G2: u64 = 0b1u64 << 14;
    pub const H2: u64 = 0b1u64 << 15;
    pub const A3: u64 = 0b1u64 << 16;
    pub const B3: u64 = 0b1u64 << 17;
    pub const C3: u64 = 0b1u64 << 18;
    pub const D3: u64 = 0b1u64 << 19;
    pub const E3: u64 = 0b1u64 << 20;
    pub const F3: u64 = 0b1u64 << 21;
    pub const G3: u64 = 0b1u64 << 22;
    pub const H3: u64 = 0b1u64 << 23;
    pub const A4: u64 = 0b1u64 << 24;
    pub const B4: u64 = 0b1u64 << 25;
    pub const C4: u64 = 0b1u64 << 26;
    pub const D4: u64 = 0b1u64 << 27;
    pub const E4: u64 = 0b1u64 << 28;
    pub const F4: u64 = 0b1u64 << 29;
    pub const G4: u64 = 0b1u64 << 30;
    pub const H4: u64 = 0b1u64 << 31;
    pub const A5: u64 = 0b1u64 << 32;
    pub const B5: u64 = 0b1u64 << 33;
    pub const C5: u64 = 0b1u64 << 34;
    pub const D5: u64 = 0b1u64 << 35;
    pub const E5: u64 = 0b1u64 << 36;
    pub const F5: u64 = 0b1u64 << 37;
    pub const G5: u64 = 0b1u64 << 38;
    pub const H5: u64 = 0b1u64 << 39;
    pub const A6: u64 = 0b1u64 << 40;
    pub const B6: u64 = 0b1u64 << 41;
    pub const C6: u64 = 0b1u64 << 42;
    pub const D6: u64 = 0b1u64 << 43;
    pub const E6: u64 = 0b1u64 << 44;
    pub const F6: u64 = 0b1u64 << 45;
    pub const G6: u64 = 0b1u64 << 46;
    pub const H6: u64 = 0b1u64 << 47;
    pub const A7: u64 = 0b1u64 << 48;
    pub const B7: u64 = 0b1u64 << 49;
    pub const C7: u64 = 0b1u64 << 50;
    pub const D7: u64 = 0b1u64 << 51;
    pub const E7: u64 = 0b1u64 << 52;
    pub const F7: u64 = 0b1u64 << 53;
    pub const G7: u64 = 0b1u64 << 54;
    pub const H7: u64 = 0b1u64 << 55;
    pub const A8: u64 = 0b1u64 << 56;
    pub const B8: u64 = 0b1u64 << 57;
    pub const C8: u64 = 0b1u64 << 58;
    pub const D8: u64 = 0b1u64 << 59;
    pub const E8: u64 = 0b1u64 << 60;
    pub const F8: u64 = 0b1u64 << 61;
    pub const G8: u64 = 0b1u64 << 62;
    pub const H8: u64 = 0b1u64 << 63;

    pub const EXTREME_ROWS: u64 = Self::A1
        | Self::B1
        | Self::C1
        | Self::D1
        | Self::E1
        | Self::F1
        | Self::G1
        | Self::H1
        | Self::A8
        | Self::B8
        | Self::C8
        | Self::D8
        | Self::E8
        | Self::F8
        | Self::G8
        | Self::H8;

    pub const SECOND_ROW: u64 =
        Self::A2 | Self::B2 | Self::C2 | Self::D2 | Self::E2 | Self::F2 | Self::G2 | Self::H2;
    pub const SEVENTH_ROW: u64 =
        Self::A7 | Self::B7 | Self::C7 | Self::D7 | Self::E7 | Self::F7 | Self::G7 | Self::H7;
}
