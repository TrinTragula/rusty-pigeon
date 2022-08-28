// Initial starting position as a FEN string
pub const START_POS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

// ALL VALUES IN CENTIPAWNS
// Value of a possibility to move
pub const SILENT_MOVE_VALUE: isize = 1;
// Value of a possibility to capture
pub const CAPTURE_VALUE: isize = 2;
// Mate value (not max size because it may cause bugs)
pub const MATE_VALUE: isize = 9999999;
// Pieces value (https://www.chessprogramming.org/Simplified_Evaluation_Function)
pub const KING_VALUE: isize = 9999999;
pub const QUEEN_VALUE: isize = 900;
pub const ROOK_VALUE: isize = 500;
pub const BISHOP_VALUE: isize = 333;
pub const KNIGHT_VALUE: isize = 323;
pub const PAWN_VALUE: isize = 100;
// Pattern values
pub const BISHOP_PAIR_VALUE: isize = 5;