use std::{
    io::stdin,
    process,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self},
};

use crate::{
    board::{fen::FenParser, models::Engine},
    constants::START_POS,
    evaluate::evaluator::Evaluate,
};

// Implementation of the UCI protocol
pub struct UCI {
    engine: Arc<Mutex<Engine>>,
    tx: Arc<Mutex<Sender<&'static str>>>,
    rx: Arc<Mutex<Receiver<&'static str>>>,
}
impl UCI {
    pub fn new() -> UCI {
        let (tx, rx) = mpsc::channel();
        UCI {
            engine: Arc::new(Mutex::new(Engine::empty())),
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn listen(&mut self) {
        let mut s = String::new();
        stdin()
            .read_line(&mut s)
            .expect("Crashed while reading from stdin");
        self.parse_command(s.trim());
    }

    fn parse_command(&mut self, command: &str) {
        let command = String::from(command.trim());
        match &command[..] {
            "uci" => {
                Self::uci();
            }
            "isready" => {
                let e = self.engine.clone();
                thread::spawn(move || loop {
                    if let Ok(e) = e.try_lock() {
                        if !e.is_searching && !e.is_configuring {
                            Self::isready();
                            break;
                        }
                    }
                });
            }
            "ucinewgame" => {
                let e = self.engine.clone();
                thread::spawn(move || {
                    Self::ucinewgame(e);
                });
            }
            "startpos" => {
                let e = self.engine.clone();
                thread::spawn(move || {
                    Self::startpos(e);
                });
            }
            "stop" => {
                self.tx.lock().unwrap().send("STOP").unwrap();
            }
            "quit" => {
                process::exit(0);
            }
            // CUSTOM COMMANDS
            "d" => {
                // Display the current state of the board
                let engine = self.engine.lock().unwrap();
                println!("{}", engine.position);
            }
            "e" => {
                // Display the current state of the board
                let mut engine = self.engine.lock().unwrap();
                println!("{}", Evaluate::static_evaluation(&mut engine));
            }
            // COMMANDS WITH ARGUMENTS
            _ => {
                let e = self.engine.clone();
                // position
                if command.starts_with("position") {
                    thread::spawn(move || {
                        Self::position(e, &command);
                    });
                }
                // go
                else if command.starts_with("go") {
                    let rx = self.rx.clone();
                    thread::spawn(move || {
                        Self::go(e, &command, rx);
                    });
                }
                /* Unknown command, ignore it, as from the specification */
            }
        }
    }

    fn uci() {
        println!("id name Rusty Pigeon");
        println!("id author TrinTragula (https://github.com/TrinTragula)");
        println!("uciok");
    }

    fn isready() {
        println!("readyok");
    }

    fn ucinewgame(e: Arc<Mutex<Engine>>) {
        let mut engine = e.lock().unwrap();
        *engine = Engine::empty();
    }

    fn startpos(e: Arc<Mutex<Engine>>) {
        let mut engine = e.lock().unwrap();
        engine.is_configuring = true;
        engine.position = FenParser::fen_to_position(START_POS);
        engine.is_configuring = false;
    }

    fn position(e: Arc<Mutex<Engine>>, command: &str) {
        let args = command.split(' ').skip(1);
        let mut engine = e.lock().unwrap();
        engine.is_configuring = true;
        let mut is_fen = false;
        let mut is_fen_parsed = false;
        for arg in args {
            if arg == "startpos" {
                engine.position = FenParser::fen_to_position(START_POS);
            } else if arg == "moves" {
                is_fen = false;
            } else if arg == "fen" {
                is_fen = true;
            } else {
                if is_fen {
                    if !is_fen_parsed {
                        let fen = command.split(' ').skip(2).collect::<Vec<&str>>().join(" ");
                        engine.position = FenParser::fen_to_position(&fen);
                        is_fen_parsed = true;
                    }
                    continue;
                } else {
                    engine.apply_algebraic_move(arg);
                }
            }
        }
        engine.is_configuring = false;
    }

    fn go(e: Arc<Mutex<Engine>>, command: &str, rx: Arc<Mutex<Receiver<&str>>>) {
        let mut engine = e.lock().unwrap();
        Evaluate::eval_for_uci(&mut engine, command, Some(rx));
    }

    pub fn bestmove(engine: &Engine) {
        let best_move = match &engine.current_best_move[0] {
            Some(m) => {
                format!("{}", m)
            }
            None => String::from("0000"),
        };
        println!("bestmove {best_move}");
    }
}
