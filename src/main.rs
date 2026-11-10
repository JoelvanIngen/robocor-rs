use crate::cli::Behaviour;
use crate::engine::Engine;
use crate::perft::perft;
use std::sync::mpsc::channel;
use std::thread::spawn;
use uci::Command;

mod board;
mod chess;
mod cli;
mod engine;
mod error;
mod eval;
mod move_gen;
mod perft;
mod search;
mod uci;

fn main() {
    match cli::parse() {
        Behaviour::Default => start(),
        Behaviour::Perft(depth) => perft(depth),
    }
}

fn start() {
    let (uci_tx, uci_rx) = channel::<String>();
    let (engine_tx, engine_rx) = channel::<Command>();
    spawn(move || Engine::new(uci_tx, engine_rx).start());
    uci::executor(engine_tx, uci_rx);
}
