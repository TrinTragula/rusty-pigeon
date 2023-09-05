const HELP_MESSAGE: &str = "
Rusty Pigeon v0.4.0, a kinda good chess engine written in rust.
Made by TrinTragula in 2022 (https://github.com/TrinTragula).

Without arguments, starts as an UCI engine.

Accepts the following flags:
   --help, -h           Prints this help message

   --check, -c          Checks if the movegen is (still) bug-free (or if it ever was)

   --perft, -p          Executes perft. Works as follows:
                          --perft {fen} {depth} [--show-moves] [--parallel]
                        Where:
                          fen             the fen of the position to start with
                          depth           how deep should the perft go
                          --show-moves    if you want to see the count by first move
                          --parallel      if you want to execute this in parallel

   --interactive, -i    Interactive board, to play agains Rusty Pigeon in the terminal. Works as follows:
                          --interactive [fen] [--auto]
                        Where:
                          fen             the fen of the position to start with
                          --auto          if you want the computer to play itself
                          --black         if you want to play as black

";

use std::{env, io::stdin, process::exit, time::Instant};

use rustypigeonlib::constants::START_POS;
use rustypigeonlib::{
    board::{
        fen::FenParser,
        models::{Engine, Side},
        utils::perft,
    },
    evaluate::evaluator::Evaluate,
    movegen::generator::{MoveGenKind, MoveGenerator},
    uci::{options::GoOptions, protocol::UCI},
};

fn main() {
    // --help
    check_flags(("--help", "-h"), &|_| {
        println!("{HELP_MESSAGE}");
    });

    // --check
    check_flags(("--check", "-c"), &|_| {
        check();
    });

    // --perft
    check_flags(("--perft", "-p"), &|args| {
        let fen = &args[2];
        let depth: u8 = args[3].trim().parse().unwrap();
        do_perft(
            fen,
            depth,
            check_flag_present("--show-moves"),
            check_flag_present("--parallel"),
        );
    });

    // --interactive
    check_flags(("--interactive", "-i"), &|args| {
        let fen = if args.len() > 3 { &args[2] } else { START_POS };
        loop_game(
            fen,
            check_flag_present("--black"),
            check_flag_present("--auto"),
        );
    });

    // Default, start in UCI engine mode
    let uci_engine = Box::leak(Box::new(UCI::new()));
    loop {
        uci_engine.listen();
    }
}

// Check if one of a set of flags is present in the command line arguments
fn check_flags(flags: (&str, &str), fun: &dyn Fn(Vec<String>) -> ()) {
    let args: Vec<String> = env::args().collect();
    if args
        .iter()
        .any(|s| s.trim() == flags.0.trim() || s.trim() == flags.1.trim())
    {
        fun(args);
    }
}

// Check if a flag is present in the cmd line arguments
fn check_flag_present(flag: &str) -> bool {
    let args: Vec<String> = env::args().collect();
    args.iter().any(|s| s.trim() == flag.trim())
}

// Clears the terminal
fn clear_terminal() {
    print!("{esc}[2J{esc}[1;1H", esc = 27 as char);
}

// Play against the engine in a simple ascii mode
fn loop_game(fen: &str, player_play_as_black: bool, auto_play: bool) {
    clear_terminal();
    println!("\nRusty Pigeon.\n");
    let mut engine = Engine::from_position(FenParser::fen_to_position(fen));
    println!("{}", engine);
    loop {
        let is_rusty_turn = match engine.position.side_to_move.0 {
            Side::WHITE => player_play_as_black,
            Side::BLACK => !player_play_as_black,
            _ => false,
        };
        if !auto_play && !is_rusty_turn {
            let moves = MoveGenerator::get_ordered_moves_by_kind(&mut engine, MoveGenKind::All);
            let mut s = String::new();
            println!("Your move: ");
            stdin().read_line(&mut s).expect("nope");
            if !moves.iter().any(|m| format!("{m}") == s.trim()) {
                println!("Invalid move, try again.");
                continue;
            }
            engine.apply_algebraic_move(&s);
        } else {
            println!("Rusty Pigeon is thinking...");
            Evaluate::search(&mut engine, GoOptions::movetime(5000), None);
            engine.apply_move(&engine.current_best_move.clone().unwrap());
        }
        clear_terminal();
        println!("{}", engine);
        println!();
    }
}

// Check if the engine correctly evaluates possible moves, printing its performance in doing so
fn do_perft(fen: &str, depth: u8, show_moves: bool, parallel: bool) {
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    let start = Instant::now();
    let result = perft(&mut sut, depth, true, show_moves, parallel);
    let duration = start.elapsed();
    println!("perft({}) = {} in {:#?}", depth, result, duration);
    exit(0);
}

// Checks that the move generation is bug free.
// To be run once every time there are some big changes in the engine
fn check() {
    let mut sut = Engine::from_position(FenParser::fen_to_position(START_POS));
    println!("Checking startpos perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 20);
    println!("Checking startpos perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 400);
    println!("Checking startpos perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 8902);
    println!("Checking startpos perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 197281);
    println!("Checking startpos perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 4865609);
    println!("Checking startpos perft(6)");
    assert_eq!(perft(&mut sut, 6, true, false, true), 119060324);

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    println!("Checking kiwipete perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 48);
    println!("Checking kiwipete perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 2039);
    println!("Checking kiwipete perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 97862);
    println!("Checking kiwipete perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 4085603);
    println!("Checking kiwipete perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 193690690);

    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    println!("Checking pos3 perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 14);
    println!("Checking pos3 perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 191);
    println!("Checking pos3 perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 2812);
    println!("Checking pos3 perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 43238);
    println!("Checking pos3 perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 674624);

    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    println!("Checking pos4 perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 6);
    println!("Checking pos4 perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 264);
    println!("Checking pos4 perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 9467);
    println!("Checking pos4 perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 422333);
    println!("Checking pos4 perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 15833292);

    let fen = "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1";
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    println!("Checking pos4_mirrored perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 6);
    println!("Checking pos4_mirrored perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 264);
    println!("Checking pos4_mirrored perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 9467);
    println!("Checking pos4_mirrored perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 422333);
    println!("Checking pos4_mirrored perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 15833292);

    let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    println!("Checking pos5 perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 44);
    println!("Checking pos5 perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 1486);
    println!("Checking pos5 perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 62379);
    println!("Checking pos5 perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 2103487);
    println!("Checking pos5 perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 89941194);

    let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    let mut sut = Engine::from_position(FenParser::fen_to_position(fen));
    println!("Checking poslast perft(1)");
    assert_eq!(perft(&mut sut, 1, true, false, true), 46);
    println!("Checking poslast perft(2)");
    assert_eq!(perft(&mut sut, 2, true, false, true), 2079);
    println!("Checking poslast perft(3)");
    assert_eq!(perft(&mut sut, 3, true, false, true), 89890);
    println!("Checking poslast perft(4)");
    assert_eq!(perft(&mut sut, 4, true, false, true), 3894594);
    println!("Checking poslast perft(5)");
    assert_eq!(perft(&mut sut, 5, true, false, true), 164075551);
    println!("Fiuuu, everything looks good! :)");
}
