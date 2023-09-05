use std::{
    cmp,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crate::movegen::generator::{MoveGenKind, MoveGenerator, MoveInfo};

use super::models::{Castling, Engine, Move, Piece, Position, Square};

pub fn algebraic_to_square(algebraic: &str) -> u64 {
    let mut square_chars = algebraic.chars();
    let column = square_chars.next().unwrap() as u32 - 'a' as u32;
    let row = square_chars.next().unwrap().to_digit(10).unwrap();
    let pos = (row - 1) * 8 + column;
    1 << pos
}

pub fn bitboard_index_to_algebraic(index: usize) -> String {
    let start_letter = b'a';
    let row = (7 - (index / 8)) as u8;
    let column = (7 - (index % 8)) as u8;
    format!("{}{}", (start_letter + column) as char, row + 1)
}

pub fn square_to_algebraic(square: u64) -> String {
    let formatted = format!("{:0>64b}", square);
    let index = formatted.chars().position(|c| c == '1').unwrap();
    bitboard_index_to_algebraic(index)
}

pub fn get_row(square: u64) -> u8 {
    let index = square.trailing_zeros();
    (index / 8) as u8
}

pub fn get_column(square: u64) -> u8 {
    let index = square.trailing_zeros();
    (index % 8) as u8
}

pub fn algebraic_to_move(alg_string: &str, pos: &Position) -> Move {
    let alg_string = alg_string.trim().trim();
    let from = algebraic_to_square(&alg_string[..2]);
    let to = algebraic_to_square(&alg_string[2..4]);
    let promotion = if alg_string.len() > 4 {
        match &alg_string[4..5] {
            "q" => Some(Piece::QUEEN),
            "r" => Some(Piece::ROOK),
            "b" => Some(Piece::BISHOP),
            "k" => Some(Piece::KNIGHT),
            _ => None,
        }
    } else {
        None
    };
    if let Some(piece) = promotion {
        return Move::Promotion(from, to, piece);
    }
    if (to == pos.state.en_passant.0)
        && ((from & pos.board.pieces[pos.side_to_move.0][Piece::PAWN].0) > 0)
    {
        return Move::EnPassant(from, to);
    }
    if (from & pos.board.pieces[pos.side_to_move.0][Piece::KING].0) > 0 {
        if (from & Square::E1 > 0) && (to & Square::G1 > 0) {
            return Move::Castle(Castling::WHITE_KING_SIDE);
        }
        if (from & Square::E1 > 0) && (to & Square::C1 > 0) {
            return Move::Castle(Castling::WHITE_QUEEN_SIDE);
        }
        if (from & Square::E8 > 0) && (to & Square::G8 > 0) {
            return Move::Castle(Castling::BLACK_KING_SIDE);
        }
        if (from & Square::E8 > 0) && (to & Square::C8 > 0) {
            return Move::Castle(Castling::BLACK_QUEEN_SIDE);
        }
    }
    Move::Normal(from, to)
}

// Execute perft on a given position
pub fn perft(e: &mut Engine, index: u8, first: bool, show_moves: bool, parallel: bool) -> usize {
    if index == 0 {
        return 0;
    }
    let mut moves = MoveGenerator::get_ordered_moves_by_kind(e, MoveGenKind::All);
    if index == 1 {
        moves.len()
    } else {
        if first && parallel {
            let tot = Arc::new(Mutex::new(0));
            let mut threads: Vec<JoinHandle<()>> = vec![];
            let tot_moves = moves.len();
            let cpus = num_cpus::get();
            let batch_length = if tot_moves / cpus == 0 {
                1
            } else {
                tot_moves / cpus
            };
            loop {
                let actual_length = cmp::min(moves.len(), batch_length);
                let thread_moves: Vec<MoveInfo> = moves.drain(0..actual_length).collect();
                if thread_moves.is_empty() {
                    break;
                }
                let new_e = e.clone();
                let tot = Arc::clone(&tot);
                threads.push(thread::spawn(move || {
                    for actual_move in thread_moves {
                        let mut new_e = new_e.clone();
                        new_e.position.apply_move(&actual_move);
                        let num = perft(&mut new_e, index - 1, false, show_moves, false);
                        new_e.position.undo_move(&actual_move);
                        *tot.lock().unwrap() += num;
                        if show_moves {
                            println!("{}: {}", actual_move.m, num);
                        }
                    }
                }));
            }
            for handle in threads {
                handle.join().unwrap()
            }
            return *tot.lock().unwrap();
        } else {
            let mut tot: usize = 0;
            for actual_move in moves {
                e.position.apply_move(&actual_move);
                tot += perft(e, index - 1, false, show_moves, false);
                e.position.undo_move(&actual_move);
            }
            tot
        }
    }
}
