use std::sync::Mutex;

use rustypigeonlib::{
    board::{fen::FenParser, models::Engine},
    constants::START_POS,
    evaluate::evaluator::Evaluate,
    uci::options::GoOptions,
};
use lazy_static::lazy_static;
use wasm_bindgen::prelude::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine> = Mutex::new(Engine::empty());
}

#[wasm_bindgen]
pub fn startpos() {
    let mut e = ENGINE.lock().unwrap();
    *e = Engine::from_position(FenParser::fen_to_position(START_POS));
}

#[wasm_bindgen]
pub fn set_pos(fen: String) {
    let mut e = ENGINE.lock().unwrap();
    *e = Engine::from_position(FenParser::fen_to_position(&fen));
}

#[wasm_bindgen]
pub fn show() -> String {
    let e = ENGINE.lock().unwrap();
    format!("{}", e.position)
}

#[wasm_bindgen]
pub fn get_move(movetime: isize) -> String {
    let mut e = ENGINE.lock().unwrap();
    Evaluate::search(
        &mut e,
        GoOptions {
            movetime: Some(movetime),
            ..GoOptions::empty()
        },
        None,
    );
    format!("{}", e.current_best_move[0].as_ref().unwrap())
}
