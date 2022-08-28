use std::{
    ops::ControlFlow,
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc, Mutex,
    },
};

use instant::Instant;
use rustc_hash::FxHashMap;

use crate::{
    board::models::{Engine, Piece, PiecePosition, Position, Side},
    constants::{
        BISHOP_VALUE, CAPTURE_VALUE, KING_VALUE, KNIGHT_VALUE, MATE_VALUE, PAWN_VALUE, QUEEN_VALUE,
        ROOK_VALUE, SILENT_MOVE_VALUE, BISHOP_PAIR_VALUE,
    },
    movegen::generator::{MoveGenKind, MoveGenerator, MoveInfo},
    uci::{options::GoOptions, protocol::UCI},
};

use super::piece_square_tables::{
    BLACK_BISHOP_SQUARE_TABLE, BLACK_KING_BEGIN_SQUARE_TABLE, BLACK_KING_END_SQUARE_TABLE,
    BLACK_KNIGHT_SQUARE_TABLE, BLACK_PAWN_SQUARE_TABLE, BLACK_QUEEN_SQUARE_TABLE,
    BLACK_ROOK_SQUARE_TABLE, WHITE_BISHOP_SQUARE_TABLE, WHITE_KING_BEGIN_SQUARE_TABLE,
    WHITE_KING_END_SQUARE_TABLE, WHITE_KNIGHT_SQUARE_TABLE, WHITE_PAWN_SQUARE_TABLE,
    WHITE_QUEEN_SQUARE_TABLE, WHITE_ROOK_SQUARE_TABLE,
};

pub struct Evaluate;
impl Evaluate {
    // todo: Evaluate pawn structures
    // todo: Incremental evaluation of pieces (each move check the change in piece value)

    pub fn eval_for_uci(
        engine: &mut Engine,
        command: &str,
        rx: Option<Arc<Mutex<Receiver<&str>>>>,
    ) {
        let options = GoOptions::parse(command);
        Self::search(engine, options, rx);
        UCI::bestmove(engine);
    }

    pub fn search(engine: &mut Engine, options: GoOptions, rx: Option<Arc<Mutex<Receiver<&str>>>>) {
        let alpha = -isize::MAX;
        let beta = isize::MAX;
        engine.is_searching = true;
        engine.current_best_move = [None, None, None, None, None, None, None, None, None, None];

        let (max_depth, time_to_move_ms) =
            GoOptions::parse_uci_options(options, engine.position.side_to_move.0, engine.position.half_move_number);
        let start = Instant::now();

        let mut prev_ordered_moves: Option<Vec<MoveInfo>> = None;

        for i in 1..max_depth + 1 {
            if !engine.is_searching {
                return;
            }

            if let ControlFlow::Break(_) =
                Self::check_if_max_time_passed(time_to_move_ms, start, engine)
            {
                return;
            }

            // Stop search on stop signal
            if let ControlFlow::Break(_) = Self::stop_on_signal(&rx, engine) {
                return;
            }

            Self::search_depth(
                engine,
                beta,
                alpha,
                i,
                &rx,
                &mut prev_ordered_moves,
                engine.position.side_to_move.0,
                time_to_move_ms,
                start,
            );
        }
        engine.is_searching = false;
    }

    fn search_depth(
        engine: &mut Engine,
        beta: isize,
        mut alpha: isize,
        depth: usize,
        rx: &Option<Arc<Mutex<Receiver<&str>>>>,
        prev_ordered_moves: &mut Option<Vec<MoveInfo>>,
        original_side: usize,
        time_to_move_ms: Option<u128>,
        start: Instant,
    ) {
        if !engine.is_searching {
            return;
        }

        let mut moves_score: FxHashMap<MoveInfo, isize> = FxHashMap::default();

        let mut moves = match prev_ordered_moves {
            Some(ordered_moves) => ordered_moves.clone(),
            None => MoveGenerator::get_ordered_moves(engine),
        };

        for m in &moves {
            if Self::check_should_exit(engine, rx, time_to_move_ms, start) {
                return;
            }

            engine.apply_move(m);

            let mut cached_score = 0;
            let cached_zobrist = engine.zobrist_table.get(&engine.position.zobrist.hash);
            let has_key = cached_zobrist.is_some();
            let write_table = if has_key {
                let cached_triplet = cached_zobrist.unwrap()[depth - 1];

                if let Some((exact_score, alpha_bound, beta_bound)) = cached_triplet {
                    // Check if the cached value is in the bound
                    if alpha >= alpha_bound && beta <= beta_bound {
                        cached_score = exact_score;
                        false
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            };

            let score = if write_table {
                -Self::alpha_beta(
                    engine,
                    -beta,
                    -alpha,
                    depth - 1,
                    depth,
                    rx,
                    time_to_move_ms,
                    start,
                )
            } else {
                cached_score
            };

            if Self::check_should_exit(engine, rx, time_to_move_ms, start) {
                engine.undo_move(&m);
                return;
            }

            if write_table {
                if !has_key {
                    engine.zobrist_table.insert(
                        engine.position.zobrist.hash,
                        [
                            Some((score, alpha, beta)),
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                            None,
                        ],
                    );
                } else {
                    let mut new_value = *engine
                        .zobrist_table
                        .get(&engine.position.zobrist.hash)
                        .unwrap();
                    new_value[depth - 1] = Some((score, alpha, beta));
                    engine
                        .zobrist_table
                        .insert(engine.position.zobrist.hash, new_value);
                }
            }
            moves_score.insert(m.clone(), score);
            engine.undo_move(&m);

            if score > alpha {
                alpha = score;
                engine.current_best_move[0] = Some(m.clone());

                let mut pv = String::from("");
                for m in &engine.current_best_move {
                    if m.is_none() {
                        break;
                    }
                    pv.push_str(&format!(" {}", m.clone().unwrap()));
                }
                let score_cp = match original_side {
                    Side::WHITE => score,
                    Side::BLACK => -score,
                    _ => 0,
                };
                println!("info score cp {score_cp} pv{pv} depth {depth}");
            }
        }

        moves.sort_by(|a, b| moves_score.get(b).cmp(&moves_score.get(a)));
        *prev_ordered_moves = Some(moves);
    }

    fn alpha_beta(
        engine: &mut Engine,
        mut alpha: isize,
        beta: isize,
        depth_left: usize,
        starting_depth: usize,
        rx: &Option<Arc<Mutex<Receiver<&str>>>>,
        time_to_move_ms: Option<u128>,
        start: Instant,
    ) -> isize {
        if depth_left == 0 {
            let quiesce_score = Self::quiesce(engine, alpha, beta);
            return quiesce_score;
        }

        let actual_depth = starting_depth - depth_left;

        let moves = MoveGenerator::get_ordered_moves(engine);
        let tot_moves = moves.len();

        let mut search_pv = true;
        for m in moves {
            if Self::check_should_exit(engine, rx, time_to_move_ms, start) {
                break;
            }

            engine.apply_move(&m);

            let mut cached_score = 0;
            let mut cached_triplet = None;
            let cached_zobrist = engine.zobrist_table.get(&engine.position.zobrist.hash);
            let has_key = cached_zobrist.is_some();
            if has_key {
                let unwrapped = cached_zobrist.unwrap();
                cached_triplet = unwrapped[depth_left - 1];
                let mut iter = depth_left;
                while cached_triplet.is_none() {
                    cached_triplet = unwrapped[iter];
                    iter += 1;
                    if iter >= 10 {
                        break;
                    }
                }
            }

            let write_table = if let Some((exact_score, alpha_bound, beta_bound)) = cached_triplet {
                // Check if the cached value is in the bound
                if alpha >= alpha_bound && beta <= beta_bound {
                    cached_score = exact_score;
                    false
                } else {
                    true
                }
            } else {
                true
            };

            let score;
            if write_table {
                if search_pv {
                    score = -Self::alpha_beta(
                        engine,
                        -beta,
                        -alpha,
                        depth_left - 1,
                        starting_depth,
                        rx,
                        time_to_move_ms,
                        start,
                    );
                } else {
                    let null_window_score = -Self::zero_width_search(
                        engine,
                        -alpha,
                        depth_left - 1,
                        starting_depth,
                        rx,
                        time_to_move_ms,
                        start,
                    );
                    if null_window_score > alpha && null_window_score < beta {
                        // re-search
                        score = -Self::alpha_beta(
                            engine,
                            -beta,
                            -alpha,
                            depth_left - 1,
                            starting_depth,
                            rx,
                            time_to_move_ms,
                            start,
                        );
                    } else {
                        score = null_window_score;
                    }
                }
            } else {
                score = cached_score;
            };

            if Self::check_should_exit(engine, rx, time_to_move_ms, start) {
                engine.undo_move(&m);
                break;
            }

            if write_table {
                if !has_key {
                    let mut value = [None; 10];
                    value[depth_left - 1] = Some((score, alpha, beta));
                    engine
                        .zobrist_table
                        .insert(engine.position.zobrist.hash, value);
                } else {
                    // todo: find a way to not read this value again
                    let mut value = *engine
                        .zobrist_table
                        .get(&engine.position.zobrist.hash)
                        .unwrap();
                    value[depth_left - 1] = Some((score, alpha, beta));
                    engine
                        .zobrist_table
                        .insert(engine.position.zobrist.hash, value);
                }
            }

            engine.undo_move(&m);

            if score >= beta {
                return beta;
            }
            if score > alpha {
                alpha = score;
                search_pv = false;

                engine.current_best_move[actual_depth] = Some(m.clone());
            }
        }
        if let Some(value) = Self::check_mate_or_stalemate(engine, tot_moves, actual_depth) {
            return value;
        }
        alpha
    }

    fn zero_width_search(
        engine: &mut Engine,
        beta: isize,
        depth_left: usize,
        starting_depth: usize,
        rx: &Option<Arc<Mutex<Receiver<&str>>>>,
        time_to_move_ms: Option<u128>,
        start: Instant,
    ) -> isize {
        if depth_left == 0 {
            let quiesce_score = Self::quiesce(engine, beta - 1, beta);
            return quiesce_score;
        }

        let actual_depth = starting_depth - depth_left;

        let moves = MoveGenerator::get_ordered_moves(engine);
        let tot_moves = moves.len();

        for m in moves {
            if Self::check_should_exit(engine, rx, time_to_move_ms, start) {
                break;
            }

            engine.apply_move(&m);

            let score = -Self::zero_width_search(
                engine,
                1 - beta,
                depth_left - 1,
                starting_depth,
                rx,
                time_to_move_ms,
                start,
            );

            if Self::check_should_exit(engine, rx, time_to_move_ms, start) {
                engine.undo_move(&m);
                break;
            }

            engine.undo_move(&m);

            if score >= beta {
                return beta;
            }
        }
        if let Some(value) = Self::check_mate_or_stalemate(engine, tot_moves, actual_depth) {
            return value;
        }
        beta - 1
    }

    fn quiesce(engine: &mut Engine, mut alpha: isize, beta: isize) -> isize {
        let score = Self::static_evaluation(engine);
        if score >= beta {
            return score;
        }
        if alpha < score {
            alpha = score;
        }

        // todo: In quiesce, search also for checks
        let moves = MoveGenerator::get_ordered_moves_by_kind(engine, MoveGenKind::OnlyCaptures);
        for m in moves {
            engine.apply_move(&m);
            let quiesce_score = -Self::quiesce(engine, -beta, -alpha);
            engine.undo_move(&m);
            if quiesce_score >= beta {
                return beta;
            }
            if quiesce_score > alpha {
                alpha = quiesce_score;
            }
        }
        alpha
    }

    pub fn static_evaluation(engine: &mut Engine) -> isize {
        if let Some(score) = engine
            .zobrist_evaluation_table
            .get(&engine.position.zobrist.hash)
        {
            return *score;
        }

        let mut result = 0;

        // Material value
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::WHITE),
            &Piece(Piece::PAWN),
            PAWN_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::WHITE),
            &Piece(Piece::ROOK),
            ROOK_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::WHITE),
            &Piece(Piece::KNIGHT),
            KNIGHT_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::WHITE),
            &Piece(Piece::BISHOP),
            BISHOP_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::WHITE),
            &Piece(Piece::QUEEN),
            QUEEN_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::WHITE),
            &Piece(Piece::KING),
            KING_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::BLACK),
            &Piece(Piece::PAWN),
            PAWN_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::BLACK),
            &Piece(Piece::ROOK),
            ROOK_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::BLACK),
            &Piece(Piece::KNIGHT),
            KNIGHT_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::BLACK),
            &Piece(Piece::BISHOP),
            BISHOP_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::BLACK),
            &Piece(Piece::QUEEN),
            QUEEN_VALUE,
        );
        result += Self::get_pieces_value(
            &engine.position.board,
            &Side(Side::BLACK),
            &Piece(Piece::KING),
            KING_VALUE,
        );

        // Mobility
        result += Self::get_mobility_values(engine);

        // Square position tables
        result += Self::get_square_table_values(&mut engine.position);

        result = match engine.position.side_to_move {
            Side(Side::WHITE) => result,
            Side(Side::BLACK) => -result,
            _ => 0,
        };
        engine
            .zobrist_evaluation_table
            .insert(engine.position.zobrist.hash, result);
        result
    }

    #[inline(always)]
    pub fn get_pieces_value(
        board: &PiecePosition,
        side: &Side,
        piece: &Piece,
        piece_value: isize,
    ) -> isize {
        let num_of_pieces = board.pieces[side.0][piece.0].0.count_ones() as isize;
        let mut value = num_of_pieces * piece_value;
        if piece.0 == Piece::BISHOP && num_of_pieces == 2 {
            value += BISHOP_PAIR_VALUE
        }
        match side {
            Side(Side::WHITE) => value,
            Side(Side::BLACK) => -value,
            _ => 0,
        }
    }

    fn get_mobility_values(engine: &mut Engine) -> isize {
        let mut result: isize = 0;

        // Enemy moves
        let enemy_moves =
            MoveGenerator::get_pseudo_legal_moves(&engine.position, &MoveGenKind::All);
        let mut enemy_mobility_score = 0isize;
        for m in enemy_moves.iter() {
            if let Some(captured_piece) = m.captured_piece {
                enemy_mobility_score += (captured_piece as isize + 2) * CAPTURE_VALUE;
            } else {
                enemy_mobility_score += SILENT_MOVE_VALUE;
            }
        }
        engine.position.side_to_move = Side(engine.position.opposite_side());
        let own_moves = MoveGenerator::get_pseudo_legal_moves(&engine.position, &MoveGenKind::All);
        engine.position.side_to_move = Side(engine.position.opposite_side());
        let mut own_mobility_score = 0isize;
        for m in own_moves.iter() {
            if let Some(captured_piece) = m.captured_piece {
                own_mobility_score += (captured_piece as isize + 1) * CAPTURE_VALUE;
            } else {
                own_mobility_score += SILENT_MOVE_VALUE;
            }
        }

        match engine.position.side_to_move {
            Side(Side::WHITE) => {
                result += own_mobility_score;
                result -= enemy_mobility_score;
            }
            Side(Side::BLACK) => {
                result -= own_mobility_score;
                result += enemy_mobility_score;
            }
            _ => {}
        };
        result
    }

    pub fn get_square_table_values(position: &mut Position) -> isize {
        let mut result: isize = 0;
        let is_end_game = Self::is_end_game(position);

        // White
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::WHITE,
            Piece::PAWN,
            WHITE_PAWN_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::WHITE,
            Piece::ROOK,
            WHITE_ROOK_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::WHITE,
            Piece::KNIGHT,
            WHITE_KNIGHT_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::WHITE,
            Piece::BISHOP,
            WHITE_BISHOP_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::WHITE,
            Piece::QUEEN,
            WHITE_QUEEN_SQUARE_TABLE,
        );
        if is_end_game {
            result += Self::get_piece_square_table_value(
                &position.board,
                Side::WHITE,
                Piece::KING,
                WHITE_KING_END_SQUARE_TABLE,
            );
        } else {
            result += Self::get_piece_square_table_value(
                &position.board,
                Side::WHITE,
                Piece::KING,
                WHITE_KING_BEGIN_SQUARE_TABLE,
            );
        }

        // Black
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::BLACK,
            Piece::PAWN,
            BLACK_PAWN_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::BLACK,
            Piece::ROOK,
            BLACK_ROOK_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::BLACK,
            Piece::KNIGHT,
            BLACK_KNIGHT_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::BLACK,
            Piece::BISHOP,
            BLACK_BISHOP_SQUARE_TABLE,
        );
        result += Self::get_piece_square_table_value(
            &position.board,
            Side::BLACK,
            Piece::QUEEN,
            BLACK_QUEEN_SQUARE_TABLE,
        );
        if is_end_game {
            result += Self::get_piece_square_table_value(
                &position.board,
                Side::BLACK,
                Piece::KING,
                BLACK_KING_END_SQUARE_TABLE,
            );
        } else {
            result += Self::get_piece_square_table_value(
                &position.board,
                Side::BLACK,
                Piece::KING,
                BLACK_KING_BEGIN_SQUARE_TABLE,
            );
        }
        result
    }

    #[inline(always)]
    pub fn get_piece_square_table_value(
        board: &PiecePosition,
        side: usize,
        piece: usize,
        table: [isize; 64],
    ) -> isize {
        let mut result = 0;
        let mut pieces = board.pieces[side][piece].0;
        while pieces > 0 {
            let square_index = pieces.trailing_zeros() as usize;
            match side {
                Side::WHITE => result += table[square_index],
                Side::BLACK => result -= table[square_index],
                _ => {}
            };
            pieces &= pieces - 1;
        }
        result
    }

    #[inline(always)]
    pub fn is_end_game(position: &mut Position) -> bool {
        // todo: Do we need a better end game definition?
        position.board.pieces[Side::WHITE][Piece::QUEEN].0 == 0
            && position.board.pieces[Side::BLACK][Piece::QUEEN].0 == 0
            && position.half_move_number >= 40
    }

    fn stop_on_signal(
        rx: &Option<Arc<Mutex<Receiver<&str>>>>,
        engine: &mut Engine,
    ) -> ControlFlow<()> {
        if rx.is_some() {
            match rx.as_ref().unwrap().lock().unwrap().try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    engine.is_searching = false;
                    return ControlFlow::Break(());
                }
                Err(TryRecvError::Empty) => {}
            }
        }
        ControlFlow::Continue(())
    }

    fn check_if_max_time_passed(
        time_to_move_ms: Option<u128>,
        start: Instant,
        engine: &mut Engine,
    ) -> ControlFlow<()> {
        if let Some(time_to_move) = time_to_move_ms {
            let duration = start.elapsed().as_millis();
            if duration > time_to_move {
                engine.is_searching = false;
                return ControlFlow::Break(());
            }
        }
        ControlFlow::Continue(())
    }

    fn check_should_exit(
        engine: &mut Engine,
        rx: &Option<Arc<Mutex<Receiver<&str>>>>,
        time_to_move_ms: Option<u128>,
        start: Instant,
    ) -> bool {
        if !engine.is_searching {
            return true;
        }
        if let ControlFlow::Break(_) = Self::stop_on_signal(rx, engine) {
            engine.is_searching = false;
            return true;
        }
        if let ControlFlow::Break(_) =
            Self::check_if_max_time_passed(time_to_move_ms, start, engine)
        {
            engine.is_searching = false;
            return true;
        }
        false
    }

    fn check_mate_or_stalemate(
        engine: &mut Engine,
        tot_moves: usize,
        actual_depth: usize,
    ) -> Option<isize> {
        if engine.is_searching && tot_moves == 0 {
            engine.position.side_to_move = Side(engine.position.opposite_side());
            let is_check = MoveGenerator::is_position_check(&mut engine.position);
            engine.position.side_to_move = Side(engine.position.opposite_side());
            if is_check {
                // That's a mate, mate
                return Some(-(MATE_VALUE - (actual_depth as isize)));
            } else {
                // That's a stale mate, return a draw
                return Some(0);
            }
        }
        None
    }
}
