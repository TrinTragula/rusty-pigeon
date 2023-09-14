use crate::{
    board::models::{Castling, Engine, Move, Piece, Position, Side, Square},
    movegen::magic::MagicBitboard,
};

lazy_static::lazy_static! {
    pub static ref MAGIC: MagicBitboard = MagicBitboard::init();
}

use super::magic::{KING_LOOKUP, KNIGHTS_LOOKUP};

#[derive(Clone, PartialEq)]
pub enum MoveGenKind {
    OnlyCaptures,
    OnlySilent,
    OnlyKing,
    All,
}

pub struct MoveGenerator;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MoveInfo {
    pub m: Move,
    pub piece: usize,
    pub captured_piece: Option<usize>,
}
impl MoveInfo {
    pub fn get_value(&self, _e: &Engine) -> isize {
        if let Move::Promotion(_, _, _) = self.m {
            // Promotions always go first
            return 9999;
        }
        if let Move::Castle(_) = self.m {
            // Castling is good
            return 99;
        }
        if self.captured_piece.is_some() {
            let mut result: isize = 100;
            // The bigger the captured piece, the better
            result += (self.captured_piece.unwrap() + 10) as isize;
            // The smaller the moving piece, the better
            result -= self.piece as isize;
            result
        } else {
            self.piece as isize
        }
    }
}

impl MoveGenerator {
    pub fn get_ordered_moves(e: &mut Engine) -> Vec<MoveInfo> {
        if e.check_draw_coditions() {
            return vec![];
        }
        let mut moves: Vec<MoveInfo> = Self::get_legal_moves(&mut e.position, &MoveGenKind::All);
        Self::sort_moves(&mut moves, e);
        moves
    }

    pub fn get_ordered_moves_by_kind(e: &mut Engine, move_gen_kind: MoveGenKind) -> Vec<MoveInfo> {
        if e.check_draw_coditions() {
            return vec![];
        }
        let mut moves: Vec<MoveInfo> = Self::get_legal_moves(&mut e.position, &move_gen_kind);

        Self::sort_moves(&mut moves, e);

        moves
    }

    fn sort_moves(moves: &mut [MoveInfo], e: &mut Engine) {
        moves.sort_by_key(|b| std::cmp::Reverse(b.get_value(e)))
    }

    fn is_legal(new_pos: &mut Position, m: &MoveInfo) -> bool {
        if m.m == Move::Castle(Castling::WHITE_KING_SIDE) {
            let king = new_pos.board.pieces[new_pos.side_to_move.0][Piece::KING].0;
            let square_index = king.trailing_zeros();
            let king_square = 1u64 << square_index;
            if Self::are_attacked_for_castling(
                new_pos,
                king_square,
                vec![Square::E1, Square::F1, Square::G1],
            ) {
                return false;
            }
        } else if m.m == Move::Castle(Castling::WHITE_QUEEN_SIDE) {
            let king = new_pos.board.pieces[new_pos.side_to_move.0][Piece::KING].0;
            let square_index = king.trailing_zeros();
            let king_square = 1u64 << square_index;
            if Self::are_attacked_for_castling(
                new_pos,
                king_square,
                vec![Square::E1, Square::D1, Square::C1],
            ) {
                return false;
            }
        } else if m.m == Move::Castle(Castling::BLACK_KING_SIDE) {
            let king = new_pos.board.pieces[new_pos.side_to_move.0][Piece::KING].0;
            let square_index = king.trailing_zeros();
            let king_square = 1u64 << square_index;
            if Self::are_attacked_for_castling(
                new_pos,
                king_square,
                vec![Square::E8, Square::F8, Square::G8],
            ) {
                return false;
            }
        } else if m.m == Move::Castle(Castling::BLACK_QUEEN_SIDE) {
            let king = new_pos.board.pieces[new_pos.side_to_move.0][Piece::KING].0;
            let square_index = king.trailing_zeros();
            let king_square = 1u64 << square_index;
            if Self::are_attacked_for_castling(
                new_pos,
                king_square,
                vec![Square::E8, Square::D8, Square::C8],
            ) {
                return false;
            }
        } else if Self::is_check(new_pos, m) {
            return false;
        }
        true
    }

    // Check if something can eat the king
    pub fn is_check(pos: &mut Position, m: &MoveInfo) -> bool {
        pos.apply_move(m);

        // Check if we just ate the opposite king (legal move, just needed for evaluation)
        let enemy_king = pos.board.pieces[pos.side_to_move.0][Piece::KING].0;
        if enemy_king == 0 {
            // We need this move to be legal for evaluation reasons
            // (otherwise we would need to add weird checks on evaluation, because here later
            // we would try to generate for the non existant opposite king)
            pos.undo_move(m);
            return true;
        }
        let result = Self::is_position_check(pos);
        pos.undo_move(m);
        result
    }

    pub fn is_position_check(pos: &mut Position) -> bool {
        let king = pos.board.pieces[pos.opposite_side()][Piece::KING].0;
        let square_index = king.trailing_zeros();
        let king_square = 1u64 << square_index;

        let moves = MoveGenerator::get_pseudo_legal_moves(pos, &MoveGenKind::OnlyKing);
        moves.iter().any(|m| match &m.m {
            Move::Normal(_, to) => to & king_square > 0,
            Move::Promotion(_, to, _) => to & king_square > 0,
            _ => false,
        })
    }

    pub fn are_attacked_for_castling(
        pos: &mut Position,
        king_square: u64,
        squares: Vec<u64>,
    ) -> bool {
        for square in squares {
            if Self::is_check(
                pos,
                &MoveInfo {
                    m: Move::Normal(king_square, square),
                    piece: Piece::KING,
                    captured_piece: None,
                },
            ) {
                return true;
            }
        }
        false
    }

    pub fn get_pseudo_legal_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];

        // Pawn
        moves.extend(Self::get_pawn_moves(pos, move_gen_kind));
        // Knights
        moves.extend(Self::get_knight_moves(pos, move_gen_kind));
        // King
        moves.extend(Self::get_king_moves(pos, move_gen_kind));
        // Rook
        moves.extend(Self::get_rook_moves(pos, move_gen_kind));
        // Bishop
        moves.extend(Self::get_bishop_moves(pos, move_gen_kind));
        // Queen
        moves.extend(Self::get_queen_moves(pos, move_gen_kind));

        moves
    }

    pub fn get_legal_moves(pos: &mut Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];

        // Pawn
        let pawn_moves: Vec<MoveInfo> = Self::get_pawn_moves(pos, move_gen_kind)
            .into_iter()
            .filter(|m| Self::is_legal(pos, m))
            .collect();
        moves.extend(pawn_moves);

        // Knights
        let knight_moves: Vec<MoveInfo> = Self::get_knight_moves(pos, move_gen_kind)
            .into_iter()
            .filter(|m| Self::is_legal(pos, m))
            .collect();
        moves.extend(knight_moves);

        // King
        let king_moves: Vec<MoveInfo> = Self::get_king_moves(pos, move_gen_kind)
            .into_iter()
            .filter(|m| Self::is_legal(pos, m))
            .collect();
        moves.extend(king_moves);

        // Rook
        let rook_moves: Vec<MoveInfo> = Self::get_rook_moves(pos, move_gen_kind)
            .into_iter()
            .filter(|m| Self::is_legal(pos, m))
            .collect();
        moves.extend(rook_moves);

        // Bishop
        let bishop_moves: Vec<MoveInfo> = Self::get_bishop_moves(pos, move_gen_kind)
            .into_iter()
            .filter(|m| Self::is_legal(pos, m))
            .collect();
        moves.extend(bishop_moves);

        // Queen
        let queen_moves: Vec<MoveInfo> = Self::get_queen_moves(pos, move_gen_kind)
            .into_iter()
            .filter(|m| Self::is_legal(pos, m))
            .collect();
        moves.extend(queen_moves);

        moves
    }

    fn get_queen_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];
        let mut queens = pos.board.pieces[pos.side_to_move.0][Piece::QUEEN].0;
        if queens == 0 {
            return moves;
        }

        let occupancy = Self::get_occupancy(pos);
        // Iterate through all the queens
        while queens > 0 {
            let square_index = queens.trailing_zeros();
            let square = 1u64 << square_index;

            // Get all the possible squares from the lookup table
            let mut move_mask = MAGIC.get_bishop_attacks(square_index as usize, occupancy)
                | MAGIC.get_rook_attacks(square_index as usize, occupancy);
            match move_gen_kind {
                MoveGenKind::All => {
                    move_mask &= !pos.board.side_pieces[pos.side_to_move.0].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::QUEEN,
                            captured_piece: Self::get_enemy_piece_in_square(
                                pos,
                                move_square,
                                false,
                            ),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlyCaptures => {
                    move_mask &= pos.board.side_pieces[pos.opposite_side()].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::QUEEN,
                            captured_piece: Self::get_enemy_piece_in_square(pos, move_square, true),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlySilent => {
                    move_mask &= !occupancy;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::QUEEN,
                            captured_piece: None,
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlyKing => {
                    move_mask &= pos.board.pieces[pos.opposite_side()][Piece::KING].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::QUEEN,
                            captured_piece: Some(Piece::KING),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
            }

            // Go to the next queen
            queens &= queens - 1
        }

        moves
    }

    fn get_bishop_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];
        let mut bishops = pos.board.pieces[pos.side_to_move.0][Piece::BISHOP].0;
        if bishops == 0 {
            return moves;
        }

        let occupancy = Self::get_occupancy(pos);
        // Check if we have any bishops
        if bishops > 0 {
            // Iterate through all the bishops
            while bishops > 0 {
                let square_index = bishops.trailing_zeros();
                let square = 1u64 << square_index;

                // Get all the possible squares from the lookup table
                let mut move_mask = MAGIC.get_bishop_attacks(square_index as usize, occupancy);
                match move_gen_kind {
                    MoveGenKind::All => {
                        move_mask &= !pos.board.side_pieces[pos.side_to_move.0].0;
                        while move_mask > 0 {
                            let move_square_index = move_mask.trailing_zeros();
                            let move_square = 1u64 << move_square_index;

                            moves.push(MoveInfo {
                                m: Move::Normal(square, move_square),
                                piece: Piece::BISHOP,
                                captured_piece: Self::get_enemy_piece_in_square(
                                    pos,
                                    move_square,
                                    false,
                                ),
                            });

                            move_mask &= move_mask - 1;
                        }
                    }
                    MoveGenKind::OnlyCaptures => {
                        move_mask &= pos.board.side_pieces[pos.opposite_side()].0;
                        while move_mask > 0 {
                            let move_square_index = move_mask.trailing_zeros();
                            let move_square = 1u64 << move_square_index;

                            moves.push(MoveInfo {
                                m: Move::Normal(square, move_square),
                                piece: Piece::BISHOP,
                                captured_piece: Self::get_enemy_piece_in_square(
                                    pos,
                                    move_square,
                                    true,
                                ),
                            });

                            move_mask &= move_mask - 1;
                        }
                    }
                    MoveGenKind::OnlySilent => {
                        move_mask &= !Self::get_occupancy(pos);
                        while move_mask > 0 {
                            let move_square_index = move_mask.trailing_zeros();
                            let move_square = 1u64 << move_square_index;

                            moves.push(MoveInfo {
                                m: Move::Normal(square, move_square),
                                piece: Piece::BISHOP,
                                captured_piece: None,
                            });

                            move_mask &= move_mask - 1;
                        }
                    }
                    MoveGenKind::OnlyKing => {
                        move_mask &= pos.board.pieces[pos.opposite_side()][Piece::KING].0;
                        while move_mask > 0 {
                            let move_square_index = move_mask.trailing_zeros();
                            let move_square = 1u64 << move_square_index;

                            moves.push(MoveInfo {
                                m: Move::Normal(square, move_square),
                                piece: Piece::BISHOP,
                                captured_piece: Some(Piece::KING),
                            });

                            move_mask &= move_mask - 1;
                        }
                    }
                }

                // Go to the next bishop
                bishops &= bishops - 1;
            }
        }

        moves
    }

    fn get_rook_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];
        let mut rooks = pos.board.pieces[pos.side_to_move.0][Piece::ROOK].0;
        if rooks == 0 {
            return moves;
        }

        let occupancy = Self::get_occupancy(pos);
        // Iterate through all the rooks
        while rooks > 0 {
            let square_index = rooks.trailing_zeros();
            let square = 1u64 << square_index;

            // Get all the possible squares from the lookup table
            let mut move_mask = MAGIC.get_rook_attacks(square_index as usize, occupancy);
            match move_gen_kind {
                MoveGenKind::All => {
                    move_mask &= !pos.board.side_pieces[pos.side_to_move.0].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::ROOK,
                            captured_piece: Self::get_enemy_piece_in_square(
                                pos,
                                move_square,
                                false,
                            ),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlyCaptures => {
                    move_mask &= pos.board.side_pieces[pos.opposite_side()].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::ROOK,
                            captured_piece: Self::get_enemy_piece_in_square(pos, move_square, true),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlySilent => {
                    move_mask &= !Self::get_occupancy(pos);
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::ROOK,
                            captured_piece: None,
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlyKing => {
                    move_mask &= pos.board.pieces[pos.opposite_side()][Piece::KING].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::ROOK,
                            captured_piece: Some(Piece::KING),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
            }

            // Go to the next rook
            rooks &= rooks - 1;
        }

        moves
    }

    fn get_king_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];
        let king = pos.board.pieces[pos.side_to_move.0][Piece::KING].0;
        // We have ONE king
        let square_index = king.trailing_zeros();
        let square = 1u64 << square_index;
        let occupancy = Self::get_occupancy(pos);

        // Get all the possible squares from the lookup table
        let mut move_mask = KING_LOOKUP[square_index as usize];
        match move_gen_kind {
            MoveGenKind::All => {
                move_mask &= !pos.board.side_pieces[pos.side_to_move.0].0;
                while move_mask > 0 {
                    let move_square_index = move_mask.trailing_zeros();
                    let move_square = 1u64 << move_square_index;

                    moves.push(MoveInfo {
                        m: Move::Normal(square, move_square),
                        piece: Piece::KING,
                        captured_piece: Self::get_enemy_piece_in_square(pos, move_square, false),
                    });

                    move_mask &= move_mask - 1;
                }
            }
            MoveGenKind::OnlyCaptures => {
                move_mask &= pos.board.side_pieces[pos.opposite_side()].0;
                while move_mask > 0 {
                    let move_square_index = move_mask.trailing_zeros();
                    let move_square = 1u64 << move_square_index;

                    moves.push(MoveInfo {
                        m: Move::Normal(square, move_square),
                        piece: Piece::KING,
                        captured_piece: Self::get_enemy_piece_in_square(pos, move_square, true),
                    });

                    move_mask &= move_mask - 1;
                }
            }
            MoveGenKind::OnlySilent => {
                move_mask &= !occupancy;
                while move_mask > 0 {
                    let move_square_index = move_mask.trailing_zeros();
                    let move_square = 1u64 << move_square_index;

                    moves.push(MoveInfo {
                        m: Move::Normal(square, move_square),
                        piece: Piece::KING,
                        captured_piece: None,
                    });

                    move_mask &= move_mask - 1;
                }
            }
            MoveGenKind::OnlyKing => {
                move_mask &= pos.board.pieces[pos.opposite_side()][Piece::KING].0;
                while move_mask > 0 {
                    let move_square_index = move_mask.trailing_zeros();
                    let move_square = 1u64 << move_square_index;

                    moves.push(MoveInfo {
                        m: Move::Normal(square, move_square),
                        piece: Piece::KING,
                        captured_piece: Some(Piece::KING),
                    });

                    move_mask &= move_mask - 1;
                }
            }
        }

        if *move_gen_kind == MoveGenKind::All || *move_gen_kind == MoveGenKind::OnlySilent {
            // Check castling
            if pos.side_to_move.0 == Side::WHITE {
                if (pos.state.castling.0 & Castling::WHITE_KING_SIDE) > 0
                    && Self::is_square_empty(occupancy, Square::F1)
                    && Self::is_square_empty(occupancy, Square::G1)
                {
                    moves.push(MoveInfo {
                        m: Move::Castle(Castling::WHITE_KING_SIDE),
                        piece: Piece::KING,
                        captured_piece: None,
                    });
                }
                if (pos.state.castling.0 & Castling::WHITE_QUEEN_SIDE) > 0
                    && Self::is_square_empty(occupancy, Square::B1)
                    && Self::is_square_empty(occupancy, Square::C1)
                    && Self::is_square_empty(occupancy, Square::D1)
                {
                    moves.push(MoveInfo {
                        m: Move::Castle(Castling::WHITE_QUEEN_SIDE),
                        piece: Piece::KING,
                        captured_piece: None,
                    });
                }
            } else {
                if (pos.state.castling.0 & Castling::BLACK_KING_SIDE) > 0
                    && Self::is_square_empty(occupancy, Square::F8)
                    && Self::is_square_empty(occupancy, Square::G8)
                {
                    moves.push(MoveInfo {
                        m: Move::Castle(Castling::BLACK_KING_SIDE),
                        piece: Piece::KING,
                        captured_piece: None,
                    });
                }
                if (pos.state.castling.0 & Castling::BLACK_QUEEN_SIDE) > 0
                    && Self::is_square_empty(occupancy, Square::B8)
                    && Self::is_square_empty(occupancy, Square::C8)
                    && Self::is_square_empty(occupancy, Square::D8)
                {
                    moves.push(MoveInfo {
                        m: Move::Castle(Castling::BLACK_QUEEN_SIDE),
                        piece: Piece::KING,
                        captured_piece: None,
                    });
                }
            }
        }

        moves
    }

    fn get_knight_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];
        let mut knights = pos.board.pieces[pos.side_to_move.0][Piece::KNIGHT].0;
        if knights == 0 {
            return moves;
        }

        // Iterate through all the knights
        while knights > 0 {
            let square_index = knights.trailing_zeros();
            let square = 1u64 << square_index;

            // Get all the possible squares from the lookup table
            let mut move_mask = KNIGHTS_LOOKUP[square_index as usize];
            match move_gen_kind {
                MoveGenKind::All => {
                    move_mask &= !pos.board.side_pieces[pos.side_to_move.0].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::KNIGHT,
                            captured_piece: Self::get_enemy_piece_in_square(
                                pos,
                                move_square,
                                false,
                            ),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlyCaptures => {
                    move_mask &= pos.board.side_pieces[pos.opposite_side()].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::KNIGHT,
                            captured_piece: Self::get_enemy_piece_in_square(pos, move_square, true),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlySilent => {
                    move_mask &= !(pos.board.side_pieces[Side::WHITE].0
                        | pos.board.side_pieces[Side::BLACK].0);
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::KNIGHT,
                            captured_piece: None,
                        });

                        move_mask &= move_mask - 1;
                    }
                }
                MoveGenKind::OnlyKing => {
                    move_mask &= pos.board.pieces[pos.opposite_side()][Piece::KING].0;
                    while move_mask > 0 {
                        let move_square_index = move_mask.trailing_zeros();
                        let move_square = 1u64 << move_square_index;

                        moves.push(MoveInfo {
                            m: Move::Normal(square, move_square),
                            piece: Piece::KNIGHT,
                            captured_piece: Some(Piece::KING),
                        });

                        move_mask &= move_mask - 1;
                    }
                }
            }
            // Go to the next knight
            knights &= knights - 1;
        }
        moves
    }

    fn get_pawn_moves(pos: &Position, move_gen_kind: &MoveGenKind) -> Vec<MoveInfo> {
        let mut moves: Vec<MoveInfo> = vec![];
        let mut pawns = pos.board.pieces[pos.side_to_move.0][Piece::PAWN].0;
        if pawns == 0 {
            return moves;
        }

        let is_white = pos.side_to_move.0 == Side::WHITE;
        let occupancy = Self::get_occupancy(pos);

        let promotions = [Piece::QUEEN, Piece::ROOK, Piece::KNIGHT, Piece::BISHOP];
        let from_row_for_promotion = if is_white {
            Square::SEVENTH_ROW
        } else {
            Square::SECOND_ROW
        };
        // Check if can move forward twice
        let row = if is_white {
            Square::SECOND_ROW
        } else {
            Square::SEVENTH_ROW
        };

        // Iterate through all the pawns
        while pawns > 0 {
            let square_index = pawns.trailing_zeros();
            let square = 1u64 << square_index;

            // Check if promotion is available
            let is_promotion = (square & from_row_for_promotion) > 0;

            if *move_gen_kind == MoveGenKind::OnlySilent || *move_gen_kind == MoveGenKind::All {
                // Check if can move forward
                let to = if is_white {
                    square.wrapping_shl(8)
                } else {
                    square.wrapping_shr(8)
                };
                let is_front_square_empty = Self::is_square_empty(occupancy, to);
                if to != 0 && is_front_square_empty {
                    if !is_promotion {
                        moves.push(MoveInfo {
                            m: Move::Normal(square, to),
                            piece: Piece::PAWN,
                            captured_piece: None,
                        });
                    } else {
                        moves.push(MoveInfo {
                            m: Move::Promotion(square, to, Piece::QUEEN),
                            piece: Piece::PAWN,
                            captured_piece: None,
                        });
                        moves.push(MoveInfo {
                            m: Move::Promotion(square, to, Piece::ROOK),
                            piece: Piece::PAWN,
                            captured_piece: None,
                        });
                        moves.push(MoveInfo {
                            m: Move::Promotion(square, to, Piece::KNIGHT),
                            piece: Piece::PAWN,
                            captured_piece: None,
                        });
                        moves.push(MoveInfo {
                            m: Move::Promotion(square, to, Piece::BISHOP),
                            piece: Piece::PAWN,
                            captured_piece: None,
                        });
                    }

                    if square & row > 0 {
                        let to2 = if is_white {
                            square.wrapping_shl(16)
                        } else {
                            square.wrapping_shr(16)
                        };
                        if Self::is_square_empty(occupancy, to2) {
                            moves.push(MoveInfo {
                                m: Move::Normal(square, to2),
                                piece: Piece::PAWN,
                                captured_piece: None,
                            });
                        }
                    }
                }
            }

            if *move_gen_kind == MoveGenKind::OnlyCaptures || *move_gen_kind == MoveGenKind::All {
                // Check captures
                let remainder = square_index % 8 == 0;
                let to1 = if is_white {
                    if remainder {
                        Square::NONE
                    } else {
                        square.wrapping_shl(7)
                    }
                } else {
                    if remainder {
                        Square::NONE
                    } else {
                        square.wrapping_shr(9)
                    }
                };
                if to1 != 0 && Self::is_square_enemy(pos, to1) {
                    if !is_promotion {
                        moves.push(MoveInfo {
                            m: Move::Normal(square, to1),
                            piece: Piece::PAWN,
                            captured_piece: Self::get_enemy_piece_in_square(pos, to1, true),
                        });
                    } else {
                        let enemy_piece = Self::get_enemy_piece_in_square(pos, to1, true);
                        moves.extend(promotions.iter().map(|&promotion| MoveInfo {
                            m: Move::Promotion(square, to1, promotion),
                            piece: Piece::PAWN,
                            captured_piece: enemy_piece,
                        }));
                    }
                }

                let remainder2 = square_index % 8 == 7;
                let to2 = if is_white {
                    if remainder2 {
                        Square::NONE
                    } else {
                        square.wrapping_shl(9)
                    }
                } else {
                    if remainder2 {
                        Square::NONE
                    } else {
                        square.wrapping_shr(7)
                    }
                };
                if to2 != 0 && Self::is_square_enemy(pos, to2) {
                    if !is_promotion {
                        moves.push(MoveInfo {
                            m: Move::Normal(square, to2),
                            piece: Piece::PAWN,
                            captured_piece: Self::get_enemy_piece_in_square(pos, to2, true),
                        });
                    } else {
                        let enemy_piece = Self::get_enemy_piece_in_square(pos, to2, true);
                        moves.extend(promotions.iter().map(|&promotion| MoveInfo {
                            m: Move::Promotion(square, to2, promotion),
                            piece: Piece::PAWN,
                            captured_piece: enemy_piece,
                        }));
                    }
                }

                // Check en passant
                if pos.state.en_passant.0 != Square::NONE
                    // && Self::is_square_empty(occupancy, pos.state.en_passant.0) shouldn't be possibile, do not check this
                    && (to1 == pos.state.en_passant.0 || to2 == pos.state.en_passant.0)
                {
                    moves.push(MoveInfo {
                        m: Move::EnPassant(square, pos.state.en_passant.0),
                        piece: Piece::PAWN,
                        captured_piece: Some(Piece::PAWN),
                    });
                }
            }

            if *move_gen_kind == MoveGenKind::OnlyKing {
                // Check captures
                let remainder = square_index % 8 == 0;
                let to1 = if is_white {
                    if remainder {
                        Square::NONE
                    } else {
                        square.wrapping_shl(7)
                    }
                } else {
                    if remainder {
                        Square::NONE
                    } else {
                        square.wrapping_shr(9)
                    }
                };
                if to1 != 0 && (pos.board.pieces[pos.opposite_side()][Piece::KING].0 & to1) > 0 {
                    if !is_promotion {
                        moves.push(MoveInfo {
                            m: Move::Normal(square, to1),
                            piece: Piece::PAWN,
                            captured_piece: Some(Piece::KING),
                        });
                    } else {
                        moves.push(MoveInfo {
                            m: Move::Promotion(square, to1, Piece::QUEEN),
                            piece: Piece::PAWN,
                            captured_piece: Some(Piece::KING),
                        });
                    }
                }

                let remainder2 = square_index % 8 == 7;
                let to2 = if is_white {
                    if remainder2 {
                        Square::NONE
                    } else {
                        square.wrapping_shl(9)
                    }
                } else {
                    if remainder2 {
                        Square::NONE
                    } else {
                        square.wrapping_shr(7)
                    }
                };
                if to2 != 0 && (pos.board.pieces[pos.opposite_side()][Piece::KING].0 & to2) > 0 {
                    if !is_promotion {
                        moves.push(MoveInfo {
                            m: Move::Normal(square, to2),
                            piece: Piece::PAWN,
                            captured_piece: Some(Piece::KING),
                        });
                    } else {
                        moves.push(MoveInfo {
                            m: Move::Promotion(square, to2, Piece::QUEEN),
                            piece: Piece::PAWN,
                            captured_piece: Some(Piece::KING),
                        });
                    }
                }
            }

            // Go to the next pawn
            pawns &= pawns - 1;
        }
        moves
    }

    fn is_square_empty(occupancy: u64, square: u64) -> bool {
        (occupancy & square) == 0
    }

    fn get_occupancy(pos: &Position) -> u64 {
        pos.board.side_pieces[Side::WHITE].0 | pos.board.side_pieces[Side::BLACK].0
    }

    fn is_square_enemy(pos: &Position, square: u64) -> bool {
        (pos.board.side_pieces[pos.opposite_side()].0 & square) > 0
    }

    fn get_enemy_piece_in_square(
        pos: &Position,
        square: u64,
        is_enemy_for_sure: bool,
    ) -> Option<usize> {
        if is_enemy_for_sure || Self::is_square_enemy(pos, square) {
            for (index, piece_board) in pos.board.pieces[pos.opposite_side()].iter().enumerate() {
                if (piece_board.0 & square) != 0 {
                    // Got it! This is the piece
                    return Some(index);
                }
            }
            None
        } else {
            None
        }
    }
}
