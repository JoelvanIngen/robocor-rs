use crate::engine::Engine;
use crate::uci::Command;
use std::sync::mpsc::channel;
use std::thread::spawn;

mod board;
mod engine;
mod error;
mod uci;

fn main() {
    let (uci_tx, uci_rx) = channel::<String>();
    let (engine_tx, engine_rx) = channel::<Command>();
    spawn(move || Engine::new(uci_tx, engine_rx).start());
    uci::executor(engine_tx, uci_rx);
}
