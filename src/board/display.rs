use crate::movegen::generator::{MoveGenerator, MoveGenKind, MoveInfo};

use super::{
    models::{Castling, Engine, Move, Piece, PiecePosition, Position, Side, Square},
    utils::square_to_algebraic,
};
use core::fmt;

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let moves = MoveGenerator::get_ordered_moves_by_kind(&mut self.clone(), MoveGenKind::All);
        let mut available_moves = format!("\nAvailable moves ({}):\n", moves.len());
        for single_move in moves.iter() {
            available_moves.push_str(&format!("{single_move} "));
        }
        write!(
            f,
            "{}{}",
            self.position, available_moves
        )
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let board = get_board(&self.board);
        let side = format!("Turn: {}", self.side_to_move);
        let castling = get_castling_info(&self.state.castling);
        let moves = get_move(self.half_move_number);
        let en_passant = get_en_passat(&self.state.en_passant);
        let since_last_capture = get_since_last_capture(self.state.since_last_capture);
        let zobrist = format!("Zobrist hash: {:#}", self.zobrist.hash);
        write!(
            f,
            "{board}\n{side}\n{moves}\t{since_last_capture}\n{castling}\t{en_passant}\n{zobrist}\n"
        )
    }
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let side = match self {
            Side(Side::WHITE) => "White",
            Side(Side::BLACK) => "Black",
            _ => "None",
        };
        write!(f, "{}", side)
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.0 == 0 {
            write!(f, "-")
        } else {
            write!(f, "{}", square_to_algebraic(self.0))
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Piece(Piece::KING) => write!(f, "K"),
            Piece(Piece::BISHOP) => write!(f, "B"),
            Piece(Piece::KNIGHT) => write!(f, "N"),
            Piece(Piece::PAWN) => write!(f, "P"),
            Piece(Piece::QUEEN) => write!(f, "Q"),
            Piece(Piece::ROOK) => write!(f, "R"),
            _ => write!(f, "?"),
        }
    }
}

impl fmt::Display for Castling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Castling(Castling::WHITE_KING_SIDE) => write!(f, "e1g1"),
            Castling(Castling::BLACK_KING_SIDE) => write!(f, "e8g8"),
            Castling(Castling::WHITE_QUEEN_SIDE) => write!(f, "e1c1"),
            Castling(Castling::BLACK_QUEEN_SIDE) => write!(f, "e8c8"),
            _ => write!(f, "0000"),
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Move::Normal(from, to) => write!(f, "{}{}", Square(*from), Square(*to)),
            Move::Promotion(from, to, piece) => write!(f, "{}{}{}", Square(*from), Square(*to), Piece(*piece)),
            Move::Castle(castling) => write!(f, "{}", Castling(*castling)),
            Move::EnPassant(from, to) => write!(f, "{}{}", Square(*from), Square(*to)),
        }
    }
}

impl fmt::Display for MoveInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.m {
            Move::Normal(from, to) => write!(f, "{}{}", Square(*from), Square(*to)),
            Move::Promotion(from, to, piece) => write!(f, "{}{}{}", Square(*from), Square(*to), Piece(*piece)),
            Move::Castle(castling) => write!(f, "{}", Castling(*castling)),
            Move::EnPassant(from, to) => write!(f, "{}{}", Square(*from), Square(*to)),
        }
    }
}

fn get_board(positon: &PiecePosition) -> String {
    let mut board = get_empty_board();
    display_pieces(positon.pieces[Side::WHITE][Piece::PAWN].0, &mut board, "P");
    display_pieces(positon.pieces[Side::BLACK][Piece::PAWN].0, &mut board, "p");
    display_pieces(
        positon.pieces[Side::WHITE][Piece::KNIGHT].0,
        &mut board,
        "N",
    );
    display_pieces(
        positon.pieces[Side::BLACK][Piece::KNIGHT].0,
        &mut board,
        "n",
    );
    display_pieces(
        positon.pieces[Side::WHITE][Piece::BISHOP].0,
        &mut board,
        "B",
    );
    display_pieces(
        positon.pieces[Side::BLACK][Piece::BISHOP].0,
        &mut board,
        "b",
    );
    display_pieces(positon.pieces[Side::WHITE][Piece::ROOK].0, &mut board, "R");
    display_pieces(positon.pieces[Side::BLACK][Piece::ROOK].0, &mut board, "r");
    display_pieces(positon.pieces[Side::WHITE][Piece::QUEEN].0, &mut board, "Q");
    display_pieces(positon.pieces[Side::BLACK][Piece::QUEEN].0, &mut board, "q");
    display_pieces(positon.pieces[Side::WHITE][Piece::KING].0, &mut board, "K");
    display_pieces(positon.pieces[Side::BLACK][Piece::KING].0, &mut board, "k");

    board = board.replace('!', "_");
    board = board.replace('?', "_");
    board = board.replace('_', " ⏤ ");
    board = board.replace('P', " ♙ ");
    board = board.replace('p', " ♟ ");
    board = board.replace('N', " ♘ ");
    board = board.replace('n', " ♞ ");
    board = board.replace('B', " ♗ ");
    board = board.replace('b', " ♝ ");
    board = board.replace('R', " ♖ ");
    board = board.replace('r', " ♜ ");
    board = board.replace('Q', " ♕ ");
    board = board.replace('q', " ♛ ");
    board = board.replace('K', " ♔ ");
    board = board.replace('k', " ♚ ");

    board.insert_str(0, "    -----------------------\n");
    board.insert_str(0, "    A  B  C  D  E  F  G  H\n");
    board.push_str("    -----------------------\n");
    board.push_str("    A  B  C  D  E  F  G  H\n");
    board
}

fn display_pieces(pieces: u64, board: &mut String, char_to_sub: &str) {
    let formatted_pieces = format!("{:0>64b}", pieces);
    for (index, bit) in formatted_pieces.chars().enumerate() {
        if bit == '1' {
            let row = (index) / 8;
            let column = (63 - index) % 8;
            let pos = row * 16 + column + 3;
            board.replace_range(pos..pos + 1, char_to_sub);
        }
    }
}

fn get_empty_board() -> String {
    let empty_row_a = (0..4).map(|_| "!?").collect::<String>();
    let empty_row_b = (0..4).map(|_| "?!").collect::<String>();
    let mut board = String::from("");
    for i in 0..8 {
        let row = if i % 2 == 0 {
            &empty_row_a
        } else {
            &empty_row_b
        };
        board.push_str(&format!("{} |", 8 - i));
        board.push_str(row);
        board.push_str(&format!(" | {}\n", 8 - i));
    }
    board
}

fn get_castling_info(castling: &Castling) -> String {
    let mut message = String::from("Castling: ");
    let Castling(value) = castling;
    if value & Castling::WHITE_QUEEN_SIDE != 0 {
        message.push_str("Q ");
    }
    if value & Castling::WHITE_KING_SIDE != 0 {
        message.push_str("K ");
    }
    if value & Castling::BLACK_QUEEN_SIDE != 0 {
        message.push_str("q ");
    }
    if value & Castling::BLACK_KING_SIDE != 0 {
        message.push_str("k ");
    }
    if value | Castling::NO_CASTLING == 0 {
        message.push('-');
    }
    message
}

fn get_move(half_move_number: usize) -> String {
    format!("Half moves: {}", half_move_number)
}

fn get_en_passat(en_passant: &Square) -> String {
    format!("En passant: {}", en_passant)
}

fn get_since_last_capture(since_last_capture: usize) -> String {
    format!("Since capture: {}", since_last_capture)
}

pub fn print_bitboard(bitboard: u64) -> String {
    let mut board = get_empty_board();
    display_pieces(bitboard, &mut board, "1");
    board = board.replace('!', "_");
    board = board.replace('?', "_");
    board 
}
